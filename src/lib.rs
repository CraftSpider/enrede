//! A crate for working with strings in encodings beyond UTF-8. The API is designed to match `std`'s
//! strings in most cases, so working with other encodings is as transparent and painless as
//! possible.
//!
//! The name, `enrede`, is a double wordplay - **En**code/**Re**code/**De**code, and Enrede means
//! 'tangled' or 'caught' in Spanish.
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

#![warn(
    elided_lifetimes_in_paths,
    missing_docs,
    clippy::cargo,
)]

macro_rules! arrvec {
    ($($elem:expr),*) => {
        {
            let mut arr = ArrayVec::new();
            $(arr.push($elem);)*
            arr
        }
    };
}

pub mod encoding;
pub mod str;
pub mod string;

pub use encoding::Encoding;
pub use str::Str;
pub use string::String;
