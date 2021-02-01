extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse::Nothing, parse_macro_input};

use self::expand::{expand, Mode};

mod expand;

#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
  parse_macro_input!(args as Nothing);
  expand(Mode::Main, parse_macro_input!(input))
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
