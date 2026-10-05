#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(auto_cfg))]
#![doc = include_str!("../README.md")]

/// Macro to create arbitrary encoding string literals at compile-time.
///
/// The macro expects a single string or C-string literal, suffixed by the shorthand representation
/// of the encoding. This is the same value returned by [`Encoding::shorthand`].
///
/// A handful of special encoding identifiers are also supported:
/// - `""utf16`, which represents the `Utf16` alias for `Utf16LE`.
/// - `""utf32`, which represents the `Utf32` alias for the native endian Utf32 variant.
///
/// C-string literals are also supported, and return a value of type [`CStr`] instead of [`Str`].
/// Unlike raw strings, C-strings support any UTF-8 character, so just use the characters directly
/// like in a normal string literal and enrede will recode them correctly.
///
/// # Examples
///
/// Normal string literals
/// ```
/// # use enrede::{str, Str};
/// let utf8 = str!("A normal UTF-8 string"utf8);
///
/// let utf16 = str!("A normal UTF-16 string"utf16);
/// ```
///
/// Characters not present in the requested encoding will fail to compile
/// ```compile_fail
/// # use enrede::str;
/// let iso = str!("A sum: ∑x"iso8859_1);
/// ```
///
/// C-String literals
/// ```
/// # use enrede::str;
/// let utf8_c = str!(c"A UTF-8 encoded C-string"utf8);
///
/// let sj_c = str!(c"A ShiftJIS encoded C-string"shiftjis);
/// ```
///
/// Encodings that contain implicit null bytes will fail to compile
/// ```compile_fail
/// # use enrede::str;
/// let utf32_c = str!(c"A UTF-32 encoded C-string?"utf32);
/// ```
#[cfg(feature = "macros")]
pub use enrede_proc::str;

#[doc(inline)]
pub use enrede_core::*;
