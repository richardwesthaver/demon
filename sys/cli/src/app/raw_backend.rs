use std::io;

use tui::{backend::Backend, buffer::Cell, layout::Rect};

pub struct RawTerminalBackend {}

impl Backend for RawTerminalBackend {
  fn clear(&mut self) -> io::Result<()> {
    Ok(())
  }

  fn hide_cursor(&mut self) -> io::Result<()> {
    Ok(())
  }

  fn show_cursor(&mut self) -> io::Result<()> {
    Ok(())
  }

  fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
    Ok((0, 0))
  }

  fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
    Ok(())
  }

  fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
  where
    I: Iterator<Item = (u16, u16, &'a Cell)>,
  {
    Ok(())
  }

  fn size(&self) -> io::Result<Rect> {
    Ok(Rect::new(0, 0, 0, 0))
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
