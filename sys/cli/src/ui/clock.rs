use std::time::{Duration, Instant};
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Button, Canvas, Dialog, LinearLayout};
use cursive::Cursive;

pub struct Watch {
    pub last_started: Instant,
    pub last_elapsed: Duration,
    pub running: bool,
}

impl Watch {
    pub fn start(&mut self) {
        self.running = true;
        self.last_started = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        self.last_elapsed
            + if self.running {
                Instant::now() - self.last_started
            } else {
                Duration::default()
            }
    }

    pub fn pause(&mut self) {
        self.last_elapsed = self.elapsed();
        self.running = false;
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.last_elapsed = Duration::default();
    }
}

// Helper function to find the stopwatch view and run a closure on it.
pub fn run<F>(f: F) -> impl Fn(&mut cursive::Cursive)
where
    F: Fn(&mut Watch),
{
    move |s| {
        s.call_on_name("stopwatch", |c: &mut Canvas<Watch>| {
            f(c.state_mut());
        });
    }
}
