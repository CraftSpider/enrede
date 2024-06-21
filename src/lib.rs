#![doc = include_str!("../README.md")]
#![warn(elided_lifetimes_in_paths, missing_docs, clippy::cargo)]
#![no_std]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

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
