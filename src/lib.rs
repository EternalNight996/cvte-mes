#![allow(
  clippy::cognitive_complexity,
  clippy::large_enum_variant,
  clippy::module_inception,
  clippy::needless_doctest_main
)]
#![warn(
  missing_debug_implementations,
  // missing_docs,
  rust_2018_idioms,
  unreachable_pub
)]
#![deny(unused_must_use)]
#![doc(test(
  no_crate_inject,
  attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//!
//! # Rust
//!```rust
//! fn main() -> e_utils::Result<()> {
//!   if cvte_mes::input::Opts::check_empty() {
//!     let _ = app::MyApp::launch()?;
//!   } else {
//!     let opts = cvte_mes::input::Opts::new(None as Option<Vec<String>>)?;
//!     let res = cvte_mes::input::api(opts)?;
//!     println!("{}", res.to_str()?);
//!   }
//!   Ok(())
//! }
//! ```

/// MACROS
#[macro_use]
mod macros;

/// Cmd input
#[cfg(feature = "input-api")]
pub mod input;
/// MES
pub mod mes;
/// Share
#[path = "./share.rs"]
pub mod share;
