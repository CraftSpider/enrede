use crate::encoding::sealed::Sealed;
use crate::encoding::{Encoding, ValidateError};
use crate::str::Str;
use arrayvec::ArrayVec;

/// The [UTF-8](https://en.wikipedia.org/wiki/UTF-8) encoding
#[non_exhaustive]
pub struct Utf8;

impl Sealed for Utf8 {}

impl Encoding for Utf8 {
    const REPLACEMENT: char = '\u{FFFD}';
    const MAX_LEN: usize = 4;

    fn shorthand() -> &'static str {
        "utf8"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        std::str::from_utf8(bytes)
            .map(|_| ())
            .map_err(|e| ValidateError {
                valid_up_to: e.valid_up_to(),
                error_len: e.error_len().map(|e| e as u8),
            })
    }

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        let mut out = [0; 4];
        dbg!(c);
        let res = c.encode_utf8(&mut out);
        dbg!(&res);
        let mut out = ArrayVec::new();
        out.extend(res.as_bytes().iter().copied());
        Some(out)
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let c = str.as_std().chars().next().unwrap();
        (c, &str[c.len_utf8()..])
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        str.as_std().is_char_boundary(idx)
    }

    fn char_len(c: char) -> usize {
        c.len_utf8()
    }
}

/// The [UTF-16](https://en.wikipedia.org/wiki/UTF-16) encoding
pub type Utf16 = Utf16LE;

/// The [UTF-16BE](https://en.wikipedia.org/wiki/UTF-16#Byte-order_encoding_schemes) encoding
#[non_exhaustive]
pub struct Utf16BE;

impl Sealed for Utf16BE {}

impl Encoding for Utf16BE {
    const REPLACEMENT: char = '\u{FFFD}';
    const MAX_LEN: usize = 4;

    fn shorthand() -> &'static str {
        "utf16be"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        let mut surrogate = false;

        for (idx, chunk) in bytes.chunks(2).enumerate() {
            if chunk.len() != 2 {
                return Err(ValidateError {
                    valid_up_to: idx * 2,
                    error_len: None,
                });
            }

            let c = u16::from_be_bytes([chunk[0], chunk[1]]);
            if !surrogate && (0xD800..0xDC00).contains(&c) {
                surrogate = true;
            } else if surrogate && (0xDC00..0xE000).contains(&c) {
                surrogate = false;
            } else if surrogate || !((..0xD800).contains(&c) || (0xE000..).contains(&c)) {
                let err_len = if surrogate && !((..0xD800).contains(&c) || (0xE000..).contains(&c))
                {
                    4
                } else {
                    2
                };
                let idx = if surrogate { idx - 1 } else { idx };
                return Err(ValidateError {
                    valid_up_to: idx * 2,
                    error_len: Some(err_len),
                });
            }
        }
        if surrogate {
            return Err(ValidateError {
                valid_up_to: bytes.len() - 2,
                error_len: None,
            });
        }

        Ok(())
    }

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        let mut out = [0; 2];
        let res = c.encode_utf16(&mut out);
        let mut out = ArrayVec::new();
        out.extend(res[0].to_be_bytes());
        if res.len() > 1 {
            out.extend(res[1].to_be_bytes());
        }
        Some(out)
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let high = u16::from_be_bytes([bytes[0], bytes[1]]);
        if (..0xD800).contains(&high) || (0xE000..).contains(&high) {
            let c = unsafe { char::from_u32_unchecked(high as u32) };
            (c, &str[2..])
        } else {
            let low = u16::from_be_bytes([bytes[2], bytes[3]]);

            let high = (high as u32 - 0xD800) * 0x400;
            let low = low as u32 - 0xDC00;
            let c = unsafe { char::from_u32_unchecked(high + low + 0x10000) };
            (c, &str[4..])
        }
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        idx % 2 == 0 && !(0xD8..0xE0).contains(&str.as_bytes()[idx + 1])
    }

    fn char_len(c: char) -> usize {
        c.len_utf16()
    }
}

/// The [UTF-16LE](https://en.wikipedia.org/wiki/UTF-16#Byte-order_encoding_schemes) encoding
#[non_exhaustive]
pub struct Utf16LE;

impl Sealed for Utf16LE {}

impl Encoding for Utf16LE {
    const REPLACEMENT: char = '\u{FFFD}';
    const MAX_LEN: usize = 4;

    fn shorthand() -> &'static str {
        "utf16le"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        let mut surrogate = false;

        for (idx, chunk) in bytes.chunks(2).enumerate() {
            if chunk.len() != 2 {
                return Err(ValidateError {
                    valid_up_to: idx * 2,
                    error_len: None,
                });
            }

            let c = u16::from_le_bytes([chunk[0], chunk[1]]);
            if !surrogate && (0xD800..0xDC00).contains(&c) {
                surrogate = true;
            } else if surrogate && (0xDC00..0xE000).contains(&c) {
                surrogate = false;
            } else if surrogate || !((..0xD800).contains(&c) || (0xE000..).contains(&c)) {
                let err_len = if surrogate && !((..0xD800).contains(&c) || (0xE000..).contains(&c))
                {
                    4
                } else {
                    2
                };
                let idx = if surrogate { idx - 1 } else { idx };
                return Err(ValidateError {
                    valid_up_to: idx * 2,
                    error_len: Some(err_len),
                });
            }
        }
        if surrogate {
            return Err(ValidateError {
                valid_up_to: bytes.len() - 2,
                error_len: None,
            });
        }

        Ok(())
    }

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        let mut out = [0; 2];
        let res = c.encode_utf16(&mut out);
        let mut out = ArrayVec::new();
        out.extend(res[0].to_le_bytes());
        if res.len() > 1 {
            out.extend(res[1].to_le_bytes());
        }
        Some(out)
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let high = u16::from_le_bytes([bytes[0], bytes[1]]);
        if (..0xD800).contains(&high) || (0xE000..).contains(&high) {
            let c = unsafe { char::from_u32_unchecked(high as u32) };
            (c, &str[2..])
        } else {
            let low = u16::from_le_bytes([bytes[2], bytes[3]]);

            let high = (high as u32 - 0xD800) * 0x400;
            let low = low as u32 - 0xDC00;
            let c = unsafe { char::from_u32_unchecked(high + low + 0x10000) };
            (c, &str[4..])
        }
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        idx % 2 == 0 && !(0xD8..0xE0).contains(&str.as_bytes()[idx])
    }

    fn char_len(c: char) -> usize {
        c.len_utf16()
    }
}

/// The [UTF-32](https://en.wikipedia.org/wiki/UTF-32) encoding
#[non_exhaustive]
pub struct Utf32;

impl Sealed for Utf32 {}

impl Encoding for Utf32 {
    const REPLACEMENT: char = '\u{FFFD}';
    const MAX_LEN: usize = 4;

    fn shorthand() -> &'static str {
        "utf32"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        for (idx, chunk) in bytes.chunks(4).enumerate() {
            if chunk.len() != 4 {
                return Err(ValidateError {
                    valid_up_to: idx * 4,
                    error_len: None,
                });
            }

            let c = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if (0xD800..0xE000).contains(&c) || (0x110000..).contains(&c) {
                return Err(ValidateError {
                    valid_up_to: idx * 4,
                    error_len: Some(4),
                });
            }
        }

        Ok(())
    }

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        Some(ArrayVec::from((c as u32).to_le_bytes()))
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let c = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let c = unsafe { char::from_u32_unchecked(c) };
        (c, &str[4..])
    }

    fn char_bound(_: &Str<Self>, idx: usize) -> bool {
        idx % 4 == 0
    }

    fn char_len(_: char) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::must_cast_slice as cast_slice;

    #[test]
    fn test_validate_utf16_le() {
        assert!(Utf16LE::validate(b"a\0b\0c\01\02\03\0").is_ok());
        assert!(Utf16LE::validate(b"A\0 \0y\0e\0e\0:\0 \0\x01\xD8\x37\xDC").is_ok());
        // dangling surrogate (after is valid char)
        assert_eq!(
            Utf16LE::validate(b"\x01\xD8a\0"),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: Some(2),
            })
        );
        // dangling surrogate (after is invalid)
        assert_eq!(
            Utf16LE::validate(b" \0\x01\xD8\x01\xD8"),
            Err(ValidateError {
                valid_up_to: 2,
                error_len: Some(4),
            })
        );
        // dangling surrogate (final byte)
        assert_eq!(
            Utf16LE::validate(b"\x01\xD8"),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: None,
            })
        );
        // dangling surrogate (final byte, valid before it)
        assert_eq!(
            Utf16LE::validate(b"a\0b\0\x01\xD8"),
            Err(ValidateError {
                valid_up_to: 4,
                error_len: None,
            })
        );
    }

    #[test]
    fn test_encode_utf16_le() {
        assert_eq!(Utf16LE::encode_char('A'), Some(arrvec![b'A', 0]));
        assert_eq!(
            Utf16LE::encode_char('𐐷'),
            Some(arrvec![0x01, 0xD8, 0x37, 0xDC])
        );
    }

    #[test]
    fn test_decode_utf16_le() {
        let str = unsafe { Str::from_bytes_unchecked(b"A\0\x01\xD8\x37\xDCb\0") };
        let (c, str) = Utf16LE::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Utf16LE::decode_char(str);
        assert_eq!(c, '𐐷');
        let (c, _) = Utf16LE::decode_char(str);
        assert_eq!(c, 'b');
    }

    #[test]
    fn test_validate_utf16_be() {
        assert!(Utf16BE::validate(b"\0a\0b\0c\01\02\03").is_ok());
        assert!(Utf16BE::validate(b"\0A\0 \0y\0e\0e\0:\0 \xD8\x01\xDC\x37").is_ok());
        // dangling surrogate (after is valid char)
        assert_eq!(
            Utf16BE::validate(b"\xD8\x01\0a"),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: Some(2),
            })
        );
        // dangling surrogate (after is invalid)
        assert_eq!(
            Utf16BE::validate(b"\0 \xD8\x01\xD8\x01"),
            Err(ValidateError {
                valid_up_to: 2,
                error_len: Some(4),
            })
        );
        // dangling surrogate (final byte)
        assert_eq!(
            Utf16BE::validate(b"\xD8\x01"),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: None,
            })
        );
        // dangling surrogate (final byte, valid before it)
        assert_eq!(
            Utf16BE::validate(b"\0a\0b\xD8\x01"),
            Err(ValidateError {
                valid_up_to: 4,
                error_len: None,
            })
        );
    }

    #[test]
    fn test_encode_utf16_be() {
        assert_eq!(Utf16BE::encode_char('A'), Some(arrvec![0, b'A']));
        assert_eq!(
            Utf16BE::encode_char('𐐷'),
            Some(arrvec![0xD8, 0x01, 0xDC, 0x37])
        );
    }

    #[test]
    fn test_decode_utf16_be() {
        let str = unsafe { Str::from_bytes_unchecked(b"\0A\xD8\x01\xDC\x37\0b") };
        let (c, str) = Utf16BE::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Utf16BE::decode_char(str);
        assert_eq!(c, '𐐷');
        let (c, _) = Utf16BE::decode_char(str);
        assert_eq!(c, 'b');
    }

    #[test]
    fn test_validate_utf32() {
        assert!(Utf32::validate(cast_slice(&['a', 'b', 'c', '1', '2', '3'])).is_ok());
        assert!(Utf32::validate(cast_slice(&['A', ' ', 'y', 'e', 'e', ':', ' ', '𐐷'])).is_ok());
        // Invalid (surrogate)
        assert_eq!(
            Utf32::validate(cast_slice(&['a' as u32, 0xD800, 'b' as u32,])),
            Err(ValidateError {
                valid_up_to: 4,
                error_len: Some(4),
            })
        );
        assert_eq!(
            Utf32::validate(cast_slice(&[0x110000])),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: Some(4),
            })
        );
    }

    #[test]
    fn test_encode_utf32() {
        assert_eq!(Utf32::encode_char('A'), Some(arrvec![b'A', 0, 0, 0]));
        assert_eq!(
            Utf32::encode_char('𐐷'),
            Some(arrvec![0x37, 0x04, 0x01, 0x00])
        );
    }

    #[test]
    fn test_decode_utf32() {
        let str = Str::from_chars(&['A', '𐐷', 'b']);
        let (c, str) = Utf32::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Utf32::decode_char(str);
        assert_eq!(c, '𐐷');
        let (c, _) = Utf32::decode_char(str);
        assert_eq!(c, 'b');
    }
}
