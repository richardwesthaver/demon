//! DemonInit - sys/core/init.rs
//!
//! Wrappers around core_macros
pub use core_macros::{main, test};

pub mod oss {
		use std::fmt::{self, Debug};
		/// This type is a token that allows code to prove demon_core::init has been called.
		/// A function can require this proof by taking `_dm: DemonInit` as an
		/// argument.
		#[derive(Copy, Clone)]
		pub struct DemonInit {    // Prevent code outside of this crate from constructing.
				_private: (),
		}

		/// Produces a proof that init has been called, without actually calling
		/// init.
		pub const unsafe fn assume_init() -> DemonInit {
				DemonInit { _private: () }
		}

		/// Produces proof that init has been called, otherwise panics
		pub fn expect_init() -> DemonInit {
				panic!("demon_core::expect_init was called, but this is not a demon build!");
		}

		impl Debug for DemonInit {
				fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						formatter.write_str("DemonInit")
				}
		}

		#[doc(hidden)]
		pub mod r#impl {
				pub use super::assume_init as perform_init;

				pub struct DestroyGuard;

				impl DestroyGuard {
						pub fn new() -> Self {
								DestroyGuard
						}
				}
		}
}
