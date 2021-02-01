//! Demon CLI
//! This crate provides command-line argument parsing.
// TODO: move app to separate crate in lib

extern crate demon_core as core;
extern crate demon_tools as tools;

mod app;
mod cons;
#[cfg(test)]
mod tests;

use std::{
  net::Ipv4Addr,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex, RwLock,
  },
  thread::{self, park_timeout},
  time::{Duration, Instant},
};

use clap::Clap;
use crossterm::{
  event::{Event, KeyCode, KeyEvent, KeyModifiers},
  terminal,
};
use tools::{
  failure,
  net::{dns::IpTable, sniffer::*, Utilization},
  os, OpenSockets, OsInputOutput,
};
use tui::backend::{Backend, CrosstermBackend};

use crate::app::{elapsed_time, RawTerminalBackend, Ui};

/// The maximum number of characters for a node name.
pub(crate) const NODE_NAME_MAX_LENGTH: usize = 64;

/// Default sub directory to store network config.
pub(crate) const DEFAULT_NETWORK_CONFIG_PATH: &'static str = "network";

/// The recommended open file descriptor limit to be configured for the process.
const RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT: u64 = 10_000;

const DISPLAY_DELTA: Duration = Duration::from_millis(1000);

/// DeMon CLI
#[derive(Clap, Debug)]
#[clap(version = "0.0.1", author = "ellis")]
pub struct Opts {
  // CORE CONFIG
  #[clap(short, long, value_name = "FILE")]
  config: Option<String>,

  // NET-MONITOR
  #[clap(flatten)]
  render_opts: RenderOpts,
  #[clap(short, long)]
  interface: Option<String>,
  #[clap(short, long)]
  raw: bool,
  #[clap(short, long)]
  no_resolve: bool,
  #[clap(short, long)]
  show_dns: bool,
  #[clap(short, long)]
  dns_server: Option<Ipv4Addr>,

  // DEMON-NET
  #[clap(long, value_name = "IP")]
  host_address: Option<String>,
  #[clap(long, value_name = "IP")]
  server_address: Option<String>,

  // AUDIO
  #[clap(long, value_name = "INT")]
  channels: Option<usize>,
}

impl Opts {
  // re-exporting functions from Clap struct for use in demon binaries
  // let self.val = Opts.val.parse();
  pub fn parse() -> Self {
    Clap::parse()
  }
}

#[derive(Clap, Debug, Clone)]
pub struct RenderOpts {
  #[clap(short, long)]
  /// Show processes table only
  processes: bool,
  #[clap(long)] // short var conflicts with `config`
  /// Show connections table only
  connections: bool,
  #[clap(short, long)]
  /// Show remote addresses table only
  addresses: bool,
  #[clap(short, long)]
  /// Show total (cumulative) usages
  total_utilization: bool,
}

pub fn try_main() -> Result<(), failure::Error> {
  use os::get_input;
  let opts = Opts::parse();
  let os_input = get_input(&opts.interface, !opts.no_resolve, &opts.dns_server)?;
  let raw_mode = opts.raw;
  if raw_mode {
    let terminal_backend = RawTerminalBackend {};
    start(terminal_backend, os_input, opts);
  } else {
    match terminal::enable_raw_mode() {
      Ok(()) => {
        let stdout = std::io::stdout();
        let terminal_backend = CrosstermBackend::new(stdout);
        start(terminal_backend, os_input, opts);
      }
      Err(_) => failure::bail!("Failed to get stdout: try using the --raw flag"),
    }
  }
  Ok(())
}

pub fn start<B>(terminal_backend: B, os_input: OsInputOutput, opts: Opts)
where
  B: Backend + Send + 'static,
{
  let running = Arc::new(AtomicBool::new(true));
  let paused = Arc::new(AtomicBool::new(false));
  let last_start_time = Arc::new(RwLock::new(Instant::now()));
  let cumulative_time = Arc::new(RwLock::new(Duration::new(0, 0)));
  let ui_offset = Arc::new(AtomicUsize::new(0));
  let dns_shown = opts.show_dns;

  let mut active_threads = vec![];

  let terminal_events = os_input.terminal_events;
  let get_open_sockets = os_input.get_open_sockets;
  let mut write_to_stdout = os_input.write_to_stdout;
  let mut dns_client = os_input.dns_client;

  let raw_mode = opts.raw;

  let network_utilization = Arc::new(Mutex::new(Utilization::new()));
  let ui = Arc::new(Mutex::new(Ui::new(
    terminal_backend,
    opts.render_opts.clone(),
  )));

  let display_handler = thread::Builder::new()
    .name("display_handler".to_string())
    .spawn({
      let running = running.clone();
      let paused = paused.clone();
      let ui_offset = ui_offset.clone();

      let network_utilization = network_utilization.clone();
      let last_start_time = last_start_time.clone();
      let cumulative_time = cumulative_time.clone();
      let ui = ui.clone();

      move || {
        while running.load(Ordering::Acquire) {
          let render_start_time = Instant::now();
          let utilization = { network_utilization.lock().unwrap().clone_and_reset() };
          let OpenSockets { sockets_to_procs } = get_open_sockets();
          let mut ip_to_host = IpTable::new();
          if let Some(dns_client) = dns_client.as_mut() {
            ip_to_host = dns_client.cache();
            let unresolved_ips = utilization
              .connections
              .keys()
              .filter(|conn| !ip_to_host.contains_key(&conn.remote_socket.ip))
              .map(|conn| conn.remote_socket.ip)
              .collect::<Vec<_>>();
            dns_client.resolve(unresolved_ips);
          }
          {
            let mut ui = ui.lock().unwrap();
            let paused = paused.load(Ordering::SeqCst);
            let ui_offset = ui_offset.load(Ordering::SeqCst);
            if !paused {
              ui.update_state(sockets_to_procs, utilization, ip_to_host);
            }
            let elapsed_time = elapsed_time(
              *last_start_time.read().unwrap(),
              *cumulative_time.read().unwrap(),
              paused,
            );

            if raw_mode {
              ui.output_text(&mut write_to_stdout);
            } else {
              ui.draw(paused, dns_shown, elapsed_time, ui_offset);
            }
          }
          let render_duration = render_start_time.elapsed();
          if render_duration < DISPLAY_DELTA {
            park_timeout(DISPLAY_DELTA - render_duration);
          }
        }
        if !raw_mode {
          let mut ui = ui.lock().unwrap();
          ui.end();
        }
      }
    })
    .unwrap();

  active_threads.push(
    thread::Builder::new()
      .name("terminal_events_handler".to_string())
      .spawn({
        let running = running.clone();
        let display_handler = display_handler.thread().clone();

        move || {
          for evt in terminal_events {
            let mut ui = ui.lock().unwrap();

            match evt {
              Event::Resize(_x, _y) => {
                if !raw_mode {
                  let paused = paused.load(Ordering::SeqCst);
                  ui.draw(
                    paused,
                    dns_shown,
                    elapsed_time(
                      *last_start_time.read().unwrap(),
                      *cumulative_time.read().unwrap(),
                      paused,
                    ),
                    ui_offset.load(Ordering::SeqCst),
                  );
                };
              }
              Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('c'),
              })
              | Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('q'),
              }) => {
                running.store(false, Ordering::Release);
                display_handler.unpark();
                match terminal::disable_raw_mode() {
                  Ok(_) => {}
                  Err(_) => println!("Error: could not disable raw input"),
                }
                break;
              }
              Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(' '),
              }) => {
                let restarting = paused.fetch_xor(true, Ordering::SeqCst);
                if restarting {
                  *last_start_time.write().unwrap() = Instant::now();
                } else {
                  let last_start_time_copy = *last_start_time.read().unwrap();
                  let current_cumulative_time_copy = *cumulative_time.read().unwrap();
                  let new_cumulative_time =
                    current_cumulative_time_copy + last_start_time_copy.elapsed();
                  *cumulative_time.write().unwrap() = new_cumulative_time;
                }

                display_handler.unpark();
              }
              Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Tab,
              }) => {
                let paused = paused.load(Ordering::SeqCst);
                let elapsed_time = elapsed_time(
                  *last_start_time.read().unwrap(),
                  *cumulative_time.read().unwrap(),
                  paused,
                );
                let table_count = ui.get_table_count();
                let new = ui_offset.load(Ordering::SeqCst) + 1 % table_count;
                ui_offset.store(new, Ordering::SeqCst);
                ui.draw(paused, dns_shown, elapsed_time, new);
              }
              _ => (),
            };
          }
        }
      })
      .unwrap(),
  );
  active_threads.push(display_handler);

  let sniffer_threads = os_input
    .network_interfaces
    .into_iter()
    .zip(os_input.network_frames.into_iter())
    .map(|(iface, frames)| {
      let name = format!("sniffing_handler_{}", iface.name);
      let running = running.clone();
      let show_dns = opts.show_dns;
      let network_utilization = network_utilization.clone();

      thread::Builder::new()
        .name(name)
        .spawn(move || {
          let mut sniffer = Sniffer::new(iface, frames, show_dns);

          while running.load(Ordering::Acquire) {
            if let Some(segment) = sniffer.next() {
              network_utilization.lock().unwrap().update(segment);
            }
          }
        })
        .unwrap()
    })
    .collect::<Vec<_>>();
  active_threads.extend(sniffer_threads);

  for thread_handler in active_threads {
    thread_handler.join().unwrap()
  }
}
