use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Error, ItemFn, Result};

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
  Main,
  Test,
}

pub fn expand(mode: Mode, mut function: ItemFn) -> Result<TokenStream> {
  if function.sig.inputs.len() > 1 {
    return Err(Error::new_spanned(
      function.sig,
      "expected one argument of type core::DemonInit",
    ));
  }

  if mode == Mode::Main && function.sig.ident != "main" {
    return Err(Error::new_spanned(
      function.sig,
      "#[demon::main] must be used on the main function",
    ));
  }

  let guard = match mode {
    Mode::Main => Some(quote! {
        if module_path!().contains("::") {
            panic!("DemonInit must be performed in the crate root on the main function");
        }
    }),
    Mode::Test => None,
  };

  let assignment = function.sig.inputs.first().map(|arg| quote!(let #arg =));
  function.sig.inputs = Punctuated::new();

  let block = function.block;

  let body = match (function.sig.asyncness.is_some(), mode) {
    (true, Mode::Test) => quote! {
        tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async #block)
    },
    (true, Mode::Main) => quote! {
        tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async #block)
    },
    (false, _) => {
      let stmts = block.stmts;
      quote! { #(#stmts)* }
    }
  };

  function.block = parse_quote!({
      #guard
      #assignment unsafe {
          demon::r#impl::perform_init()
      };
      let destroy_guard = unsafe { demon::r#impl::DestroyGuard::new() };
      #body
  });

  function.sig.asyncness = None;

  if mode == Mode::Test {
    function.attrs.push(parse_quote!(#[test]));
  }

  Ok(quote!(#function))
}
