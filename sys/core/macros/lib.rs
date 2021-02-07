mod expand;
use expand::{expand, Mode};
use proc_macro::TokenStream;
use syn::parse::Nothing;
use syn::parse_macro_input;
/// Expands from:
///
/// #[demon_core::main]
/// fn main(dm: DemonInit) {
///  ...
/// }
///
/// to:
///
/// fn main() {
///   let dm: DemonInit = demon_core::r#impl::perform_init();
///   ...
/// }
///
/// if async, also adds a #[tokio: main] attribute
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
  parse_macro_input!(args as Nothing);
  expand(Mode::Main, parse_macro_input!(input))
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// provides same functionality as 'main' but for #[test] and #[tokio::test]
#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
  parse_macro_input!(args as Nothing);
  expand(Mode::Test, parse_macro_input!(input))
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

// #[proc_macro_attribute]
// pub fn compat_test(args: TokenStream, input: TokenStream) -> TokenStream {
//     parse_macro_input!(args as Nothing);
//     expand(Mode::CompatTest, parse_macro_input!(input))
//         .unwrap_or_else(|err| err.to_compile_error())
//         .into()
// }
