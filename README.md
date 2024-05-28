# Enrede

![Crates.io Version](https://img.shields.io/crates/v/enrede)
![Crates.io License](https://img.shields.io/crates/l/enrede)
![Actions Badge](https://github.com/craftspider/enrede/actions/workflows/ci.yml/badge.svg)

An easy-to-use string encoding library, providing an interface similar to `str`/`String`, for
working with strings in encodings beyond UTF-8. Due to the API being based on `std`, working with other
encodings is as transparent and painless as possible.

The name, `enrede`, is a double wordplay - **En**code/**Re**code/**De**code, and Enrede means
'tangled' or 'caught' in Spanish.

## Features

- `Str<E>` and `String<E>` types, equivalent to `str` and `std::string::String`,
  but generic over encoding.
- `CStr<E>` and `CString<E>` types, equivalent to `std::ffi::CStr` and `std::ffi::CString`,
  but generic over encoding.
- `Encoding` trait with support for lower-level direct encoding/recoding into slices
- `no_std` support

## Planned Features

These features are not yet supported, but are planned for a future version:

- Dynamically encoded strings
- Extended methods for encodings following certain properties:
  - Constant length encodings
- More encodings
  - Shift-JIS
  - Big5
  - ISO/IEC 8859-1
- More methods on strings and C-strings
- Performance benchmarking and guaranteed high-throughput

## `no_std` Support

By default, the `std` and `alloc` features are enabled. By using `default-features = false`
in your `Cargo.toml`, you can disable these features. When `std` is disabled, this crate
is `no_std`. When the `alloc` feature is disabled, the crate won't use `alloc`, and any types
or functions requiring allocation will be disabled (For example [`String<E>`]).

## Limitations

Currently, it is assumed that all supported encodings are subsets of the Unicode character set.
