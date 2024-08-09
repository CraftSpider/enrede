# Changelog

# [0.1.2] - 2024-08-08

### Added

- `encoding::AlwaysValid`, for encodings with no invalid byte patterns
  - `Str::from_bytes_infallible{,_mut}` - infallible variant of `from_bytes{,_mut}`
  - `String::from_bytes_infallible` - infallible variant of `from_bytes`
  - `CStr::from_bytes_{with,til}_nul_valid{,_mut}` - no encoding validation variants of
    equivalent methods.
  - `CString::new_valid` - no encoding validation variant of `new`
- Mutable `CStr` creation methods
  - `from_bytes_with_nul_mut`
  - `from_bytes_til_nul_mut`
- `rand` feature, allows encodings to also function as distributions to generate
  characters valid for that encoding.
- Added benchmarks
- New Encodings:
  - JIS X 0201
  - JIS X 0208
  - Mac Roman

### Fixed

- Win1252 encoding validation mistakenly banned 0x82 instead of 0x81, this has been fixed
- Win1251, Win1252, and JisX0201 had incorrect `char_len` implementations. They will now return the correct values.

### Changed

- Updated README.md, replace lib.rs docs with README.md

# [0.1.1] - 2024-05-28

### Added

- `no_std` support
  - `Str` and `CStr` are always available
  - `String` and `CString` are available on `alloc` feature
- `CString<E>` - encoding-specific `CString` equivalent type
- `CStr<E>` - encoding-specific `CStr` equivalent type
  - Unlike `CStr`, `CStr<E>` will deref to `Str<E>`
- New Encodings:
  - ISO/IEC 8859-2
  - ISO/IEC 8859-15

# [0.1.0] - 2024-05-25

Initial release. Adds core types and the most common encodings.

### Added

- `Encoding` - trait representing a generic encoding such as UTF-8 or Windows-1252
- `String<E>` - encoding-specific `String` equivalent type
  - `String<Utf8>` is free to convert from an `std::string::String`
- `Str<E>` - encoding specific `str` type
  - `Str<Utf8>` is free to convert from a `str`
- New Encodings:
  - ASCII
  - ASCII Extended
    - Variant of ASCII that supports all byte values
  - UTF{8, 16LE, 16BE, 32}
  - Windows-{1251, 1252}
  - Windows-1252 Loose
    - Variant of 1252 that supports all byte values
