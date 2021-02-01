#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

// use 'wasm_bindgen_test_configure!(run_in_browser);' to run browser tests

#[wasm_bindgen_test]
fn pass() {
  assert_eq!(1 + 1, 2);
}
