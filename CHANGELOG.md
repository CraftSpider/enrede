# Changelog

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
