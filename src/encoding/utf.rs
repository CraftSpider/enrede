use crate::encoding::sealed::Sealed;
use crate::encoding::{Encoding, NullTerminable, ValidateError};
use crate::str::Str;
use arrayvec::ArrayVec;
#[cfg(feature = "rand")]
use rand::{distributions::Distribution, Rng};

/// The [UTF-8](https://en.wikipedia.org/wiki/UTF-8) encoding
#[non_exhaustive]
#[derive(Default)]
pub struct Utf8;

impl Sealed for Utf8 {}

impl Encoding for Utf8 {
    const REPLACEMENT: char = '\u{FFFD}';
    const MAX_LEN: usize = 4;
    type Bytes = ArrayVec<u8, 4>;

    fn shorthand() -> &'static str {
        "utf8"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        core::str::from_utf8(bytes)
            .map(|_| ())
            .map_err(|e| ValidateError {
                valid_up_to: e.valid_up_to(),
                error_len: e.error_len().map(|e| e as u8),
            })
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        let mut out = [0; 4];
        let res = c.encode_utf8(&mut out);
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

impl NullTerminable for Utf8 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Utf8 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        rng.gen::<char>()
    }
}

/// The [UTF-16](https://en.wikipedia.org/wiki/UTF-16) encoding
pub type Utf16 = Utf16LE;

#[derive(PartialEq, Eq)]
enum Kind {
    Char,
    High,
    Low,
}

impl Kind {
    fn of(c: u16) -> Kind {
        match c {
            ..=0xD7FF => Kind::Char,
            0xD800..=0xDBFF => Kind::High,
            0xDC00..=0xDFFF => Kind::Low,
            0xE000.. => Kind::Char,
        }
    }
}

macro_rules! utf16_impl {
    (
        $name:ident,
        $shorthand:literal,
        $method_from:ident,
        $method_to:ident,
        $idx_add:literal,
        $docname:literal,
    ) => {
        #[doc = "The ["]
        #[doc = $docname]
        #[doc = "](https://en.wikipedia.org/wiki/UTF-16#Byte-order_encoding_schemes) encoding"]
        #[non_exhaustive]
        #[derive(Default)]
        pub struct $name;

        impl Sealed for $name {}

        impl Encoding for $name {
            const REPLACEMENT: char = '\u{FFFD}';
            const MAX_LEN: usize = 4;
            type Bytes = ArrayVec<u8, 4>;

            fn shorthand() -> &'static str {
                $shorthand
            }

            fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
                let chunks = bytes.chunks_exact(2);

                let error = if let [_] = chunks.remainder() {
                    Some(ValidateError {
                        valid_up_to: bytes.len() - 1,
                        error_len: None,
                    })
                } else {
                    None
                };

                // `get_unchecked` is the same speed
                // `try_fold` variant is significantly slower
                let mut surrogate = false;
                for (idx, chunk) in chunks.enumerate() {
                    let c = u16::$method_from([chunk[0], chunk[1]]);
                    let kind = Kind::of(c);

                    if !surrogate && kind == Kind::High {
                        surrogate = true;
                    } else if surrogate && kind == Kind::Low {
                        surrogate = false;
                    } else if surrogate || kind != Kind::Char {
                        let err_len = if surrogate && kind != Kind::Char {
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

                match error {
                    Some(err) => Err(err),
                    None => Ok(()),
                }
            }

            fn encode_char(c: char) -> Option<Self::Bytes> {
                let mut out = [0; 2];
                let res = c.encode_utf16(&mut out);
                let mut out = ArrayVec::new();
                out.extend(res[0].$method_to());
                if res.len() > 1 {
                    out.extend(res[1].$method_to());
                }
                Some(out)
            }

            fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
                let bytes = str.as_bytes();
                let high = u16::$method_from([bytes[0], bytes[1]]);
                if (..0xD800).contains(&high) || (0xE000..).contains(&high) {
                    // SAFETY: We just confirmed `high` is not in the surrogate range, and is thus a valid
                    //         `char`.
                    let c = unsafe { char::from_u32_unchecked(high as u32) };
                    (c, &str[2..])
                } else {
                    let low = u16::$method_from([bytes[2], bytes[3]]);

                    let high = (high as u32 - 0xD800) * 0x400;
                    let low = low as u32 - 0xDC00;
                    // SAFETY: Str is valid UTF-16, as such, all surrogate pairs will produce a valid `char`
                    let c = unsafe { char::from_u32_unchecked(high + low + 0x10000) };
                    (c, &str[4..])
                }
            }

            fn char_bound(str: &Str<Self>, idx: usize) -> bool {
                idx % 2 == 0 && !(0xD8..0xE0).contains(&str.as_bytes()[idx + $idx_add])
            }

            fn char_len(c: char) -> usize {
                c.len_utf16()
            }
        }

        #[cfg(feature = "rand")]
        impl Distribution<char> for $name {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
                rng.gen::<char>()
            }
        }
    };
}

utf16_impl!(
    Utf16BE,
    "utf16be",
    from_be_bytes,
    to_be_bytes,
    1,
    "UTF-16BE",
);

utf16_impl!(
    Utf16LE,
    "utf16le",
    from_le_bytes,
    to_le_bytes,
    0,
    "UTF-16LE",
);

/// The [UTF-32](https://en.wikipedia.org/wiki/UTF-32) encoding
#[non_exhaustive]
#[derive(Default)]
pub struct Utf32;

impl Sealed for Utf32 {}

impl Encoding for Utf32 {
    const REPLACEMENT: char = '\u{FFFD}';
    const MAX_LEN: usize = 4;
    type Bytes = [u8; 4];

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

            let c = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if (0xD800..0xE000).contains(&c) || (0x0011_0000..).contains(&c) {
                return Err(ValidateError {
                    valid_up_to: idx * 4,
                    error_len: Some(4),
                });
            }
        }

        Ok(())
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        Some((c as u32).to_le_bytes())
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let c = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        // SAFETY: Str<Utf32> is guaranteed to contain valid `char` values
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

#[cfg(feature = "rand")]
impl Distribution<char> for Utf32 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        rng.gen::<char>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::must_cast_slice as cast_slice;

    #[allow(clippy::octal_escapes)]
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
        let mut expect = ArrayVec::new();
        expect.extend([b'A', 0]);
        assert_eq!(Utf16LE::encode_char('A'), Some(expect));
        assert_eq!(
            Utf16LE::encode_char('𐐷'),
            Some(ArrayVec::from([0x01, 0xD8, 0x37, 0xDC]))
        );
    }

    #[test]
    fn test_decode_utf16_le() {
        // SAFETY: This test data is guaranteed valid
        let str = unsafe { Str::from_bytes_unchecked(b"A\0\x01\xD8\x37\xDCb\0") };
        let (c, str) = Utf16LE::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Utf16LE::decode_char(str);
        assert_eq!(c, '𐐷');
        let (c, _) = Utf16LE::decode_char(str);
        assert_eq!(c, 'b');
    }

    #[allow(clippy::octal_escapes)]
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
        let mut expect = ArrayVec::new();
        expect.extend([0, b'A']);
        assert_eq!(Utf16BE::encode_char('A'), Some(expect));
        assert_eq!(
            Utf16BE::encode_char('𐐷'),
            Some(ArrayVec::from([0xD8, 0x01, 0xDC, 0x37]))
        );
    }

    #[test]
    fn test_decode_utf16_be() {
        // SAFETY: This test data is guaranteed valid
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
            Utf32::validate(cast_slice(&[0x0011_0000])),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: Some(4),
            })
        );
    }

    #[test]
    fn test_encode_utf32() {
        assert_eq!(Utf32::encode_char('A'), Some([b'A', 0, 0, 0]));
        assert_eq!(Utf32::encode_char('𐐷'), Some([0x37, 0x04, 0x01, 0x00]));
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
