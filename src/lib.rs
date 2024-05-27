//! A crate for working with strings in encodings beyond UTF-8. The API is designed to match `std`'s
//! strings in most cases, so working with other encodings is as transparent and painless as
//! possible.
//!
//! The name, `enrede`, is a double wordplay - **En**code/**Re**code/**De**code, and Enrede means
//! 'tangled' or 'caught' in Spanish.
//!
//! ## `no_std` Support
//!
//! By default, the `std` and `alloc` features are enabled. By using `default-features = false`
//! in your `Cargo.toml`, you can disable these features. When `std` is disabled, this crate
//! is `no_std`. When the `alloc` feature is disabled, the crate won't use `alloc`, and any types
//! or functions requiring allocation will be disabled (For example [`String<E>`]).
//!
//! ## Limitations
//!
//! Currently, it is assumed that all supported encodings are subsets of the Unicode character set.
//!
//! ## TODO
//!
//! These features are not yet supported, but are planned for a future version:
//! - Dynamically encoded strings
//! - Extended methods for encodings following certain properties:
//!   - Constant length encodings
//!   - Encodings with no invalid byte sequences
//! - More encodings
//!   - Shift-JIS
//!   - Big5
//!   - ISO/IEC 8859-1

#![warn(elided_lifetimes_in_paths, missing_docs, clippy::cargo)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

pub mod cstr;
#[cfg(feature = "alloc")]
pub mod cstring;
pub mod encoding;
pub(crate) mod err;
pub mod str;
#[cfg(feature = "alloc")]
pub mod string;
pub(crate) mod utils;

pub use cstr::CStr;
#[cfg(feature = "alloc")]
pub use cstring::CString;
pub use encoding::Encoding;
pub use str::Str;
#[cfg(feature = "alloc")]
pub use string::String;
