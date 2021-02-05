pub mod clock;
pub mod keylog;
pub mod tree;
pub mod calendar;
use cursive::view::Position;
use cursive::views::LayerPosition;
use cursive::Cursive;

/// Moves top layer by the specified amount
pub fn move_top(c: &mut Cursive, x_in: isize, y_in: isize) {
  // Step 1. Get the current position of the layer.
  let s = c.screen_mut();
  let l = LayerPosition::FromFront(0);

  // Step 2. add the specifed amount
  let pos = s.offset().saturating_add((x_in, y_in));

  // convert the new x and y into a position
  let p = Position::absolute(pos);

  // Step 3. Apply the new position
  s.reposition_layer(l, p);
}
