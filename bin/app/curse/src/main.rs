use demon_cli::cursive::{
  self,
  event::Key,
  menu::MenuTree,
  traits::*,
  views::{Button, Canvas, CircularFocus, Dialog, HideableView, LinearLayout, TextView},
  Cursive,
};
use demon_cli::ui::{
  clock::{run, Watch},
  move_top,
  tree::{expand_tree, Placement, TreeEntry, TreeView},
};
use demon_cli::{DMC_BANNER, DMZ_BANNER};
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

fn main() {
  cursive::logger::init();
  // Create TreeView with initial working directory
  let mut tree = TreeView::<TreeEntry>::new();
  let path = env::current_dir().expect("Working directory missing.");

  tree.insert_item(
    TreeEntry {
      name: path.file_name().unwrap().to_str().unwrap().to_string(),
      dir: Some(path.clone()),
    },
    Placement::After,
    0,
  );

  expand_tree(&mut tree, 0, &path);

  // Lazily insert directory listings for sub nodes
  tree.set_on_collapse(|siv: &mut Cursive, row, is_collapsed, children| {
    if !is_collapsed && children == 0 {
      siv.call_on_name("tree", move |tree: &mut TreeView<TreeEntry>| {
        if let Some(dir) = tree.borrow_item(row).unwrap().dir.clone() {
          expand_tree(tree, row, &dir);
        }
      });
    }
  });

  let counter = AtomicUsize::new(1);

  // Setup Cursive
  let mut siv = cursive::default();
  siv.add_global_callback('q', |s| s.quit());
  siv.load_toml(include_str!("../dms.toml")).unwrap();
  siv.add_layer(
    // Most views can be configured in a chainable way
    LinearLayout::horizontal()
      .child(
        Dialog::around(TextView::new(DMZ_BANNER))
          .title("Curse!")
          .with_name("main"),
      )
      .child(
        Dialog::new()
          .title("Stopwatch")
          .content(
            LinearLayout::horizontal()
              .child(
                Canvas::new(Watch {
                  last_started: Instant::now(),
                  last_elapsed: Duration::default(),
                  running: true,
                })
                .with_draw(|s, printer| {
                  printer.print((0, 1), &format!("{:.2?}", s.elapsed()));
                })
                .with_name("stopwatch")
                .fixed_size((8, 3)),
              )
              .child(
                LinearLayout::vertical()
                  .child(Button::new("Start", run(Watch::start)))
                  .child(Button::new("Pause", run(Watch::pause)))
                  .child(Button::new("Stop", run(Watch::stop))),
              ),
          )
          .dismiss_button("Close")
          .h_align(cursive::align::HAlign::Right),
      ),
  );

  siv
    .menubar()
    // We add a new "File" tree
    .add_subtree(
      "File",
      MenuTree::new()
        // Trees are made of leaves, with are directly actionable...
        .leaf("New", move |s| {
          // Here we use the counter to add an entry
          // in the list of "Recent" items.
          let i = counter.fetch_add(1, Ordering::Relaxed);
          let filename = format!("New {}", i);
          s.menubar()
            .find_subtree("File")
            .unwrap()
            .find_subtree("Recent")
            .unwrap()
            .insert_leaf(0, filename, |_| ());

          s.add_layer(Dialog::info("New file!"));
        })
        // ... and of sub-trees, which open up when selected.
        .subtree(
          "Recent",
          // The `.with()` method can help when running loops
          // within builder patterns.
          MenuTree::new().with(|tree| {
            for i in 1..100 {
              // We don't actually do anything here,
              // but you could!
              tree.add_leaf(format!("Item {}", i), |_| ())
            }
          }),
        )
        // Delimiter are simple lines between items,
        // and cannot be selected.
        .delimiter()
        .with(|tree| {
          for i in 1..10 {
            tree.add_leaf(format!("Option {}", i), |_| ());
          }
        }),
    )
    .add_subtree(
      "Help",
      MenuTree::new()
        .subtree(
          "Help",
          MenuTree::new()
            .leaf("General", |s| s.add_layer(Dialog::info("Help message!")))
            .leaf("Online", |s| {
              let text = "Google it yourself!\n\
                                        Kids, these days...";
              s.add_layer(Dialog::info(text))
            }),
        )
        .leaf("About", |s| s.add_layer(Dialog::info("Cursive v0.0.0"))),
    )
    .add_delimiter()
    .add_leaf("Quit", |s| s.quit());

  // When `autohide` is on (default), the menu only appears when active.
  // Turning it off will leave the menu always visible.
  // Try uncommenting this line!

  // siv.set_autohide_menu(false);

  siv.add_global_callback(Key::Esc, |s| s.select_menubar());
  // Next Gen FPS Controls for current layer.
  siv.add_global_callback('p', |s| move_top(s, 0, -5));
  siv.add_global_callback('b', |s| move_top(s, -5, 0));
  siv.add_global_callback('n', |s| move_top(s, 0, 5));
  siv.add_global_callback('f', |s| move_top(s, 5, 0));
  siv.add_global_callback('d', cursive::Cursive::toggle_debug_console);

  //    siv.add_layer(KeyCodeView::new(10).full_width().fixed_height(10));
  //    siv.add_layer(Dialog::around(tree.with_name("tree").scrollable()).title("File View"));
  siv.set_fps(20); // big balls
  siv.run();
}
