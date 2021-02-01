use std::{
  convert::From,
  fs,
  io::{Error as IOError, Write},
  path::Path,
  result::Result,
  string::FromUtf8Error,
};

use orgize::{
  export::{DefaultHtmlHandler, HtmlHandler},
  Element, Org, ParseConfig,
};
use slugify::slugify;

#[derive(Debug)]
pub enum OrgError {
  IO(IOError),
  Heading,
  Utf8(FromUtf8Error),
}

impl From<IOError> for OrgError {
  fn from(err: IOError) -> Self {
    OrgError::IO(err)
  }
}

impl From<FromUtf8Error> for OrgError {
  fn from(err: FromUtf8Error) -> Self {
    OrgError::Utf8(err)
  }
}

#[derive(Default)]
struct OrgHtmlHandler(DefaultHtmlHandler);

impl HtmlHandler<OrgError> for OrgHtmlHandler {
  fn start<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), OrgError> {
    if let Element::Title(title) = element {
      if title.level > 6 {
        return Err(OrgError::Heading);
      } else {
        write!(
          w,
          "<h{0}><a id=\"{1}\" href=\"#{1}\">",
          title.level,
          slugify!(&title.raw),
        )?;
      }
    } else {
      // fallthrough to default handler
      self.0.start(w, element)?;
    }
    Ok(())
  }

  fn end<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), OrgError> {
    if let Element::Title(title) = element {
      write!(w, "</a></h{}>", title.level)?;
    } else {
      self.0.end(w, element)?;
    }
    Ok(())
  }
}

pub fn org_export_html(file: &str, output: &str) -> Result<(), OrgError> {
  let contents = String::from_utf8(fs::read(&file)?)?;
  println!("exporting file: {:?}", &contents);
  let mut writer = Vec::new();
  let mut handler = OrgHtmlHandler::default();

  Org::parse(&contents).write_html_custom(&mut writer, &mut handler)?;
  fs::write(output, String::from_utf8(writer)?);

  Ok(())
}
