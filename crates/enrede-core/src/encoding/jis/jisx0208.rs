use crate::encoding::jis::x0208_tables;
use crate::encoding::sealed::Sealed;
use crate::encoding::{Enc, NullTerminable, ValidateError};
use crate::{Encoding, Str};
use arrayvec::ArrayVec;
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

/// The [JIS X 0208](https://en.wikipedia.org/wiki/JIS_X_0208) encoding.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct JisX0208;

impl Sealed for JisX0208 {}

impl Encoding for JisX0208 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 2;
    type Bytes = ArrayVec<u8, 2>;

    fn shorthand() -> &'static str {
        "jisx0208"
    }

    fn dyn_enc() -> Enc {
        Enc::JisX0208
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        let mut row = 0;
        for (idx, b) in bytes.iter().enumerate() {
            if *b >= 0x80 {
                return Err(ValidateError {
                    valid_up_to: idx,
                    error_len: Some(1),
                });
            } else if row == 0 {
                // Tables with no valid characters - fast path
                if ((0x29..0x30).contains(b) && *b != 0x2D) || (0x75..0x7F).contains(b) {
                    return Err(ValidateError {
                        valid_up_to: idx,
                        error_len: Some(2),
                    });
                } else if (0x21..0x7F).contains(b) {
                    row = *b - 0x20;
                }
                // Characters in range 0..0x20 are ASCII control codes
            } else if row != 0 {
                if !(0x21..0x7F).contains(b)
                    || x0208_tables::DECODE_MAP_0208[(row - 1) as usize][(*b - 0x21) as usize]
                        == '�'
                {
                    return Err(ValidateError {
                        valid_up_to: idx - 1,
                        error_len: Some(2),
                    });
                }
                row = 0;
            }
        }
        Ok(())
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if c as u32 <= 0x20 || c as u32 == 0x7F {
            Some(ArrayVec::from_iter([c as u8]))
        } else {
            let (row, col) = x0208_tables::ENCODE_MAP_0208[&c];
            Some(ArrayVec::from([row as u8 + 0x21, col as u8 + 0x21]))
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let first = bytes[0];
        if (..0x21).contains(&first) || first == 0x7F {
            (char::from(first), unsafe { str.get_unchecked(1..) })
        } else {
            let second = bytes[1];
            let (row, col) = (first - 0x21, second - 0x21);
            let c = x0208_tables::DECODE_MAP_0208[row as usize][col as usize];
            (c, unsafe { str.get_unchecked(2..) })
        }
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        let bytes = str.as_bytes();
        let first = bytes[0];
        // Control code bytes, space, and del - always single-byte, never used as a second byte
        if (..0x21).contains(&first) || first == 0x7F {
            true
        } else {
            // Otherwise, first and second bytes look the same - iterate to here
            for (idx2, _) in str.char_indices() {
                if idx == idx2 {
                    return true;
                } else if idx < idx2 {
                    return false;
                }
            }
            false
        }
    }

    fn char_len(c: char) -> usize {
        if (..0x21).contains(&(c as u32)) || c as u32 == 0x7F {
            1
        } else if x0208_tables::DECODE_MAP_0208
            .iter()
            .any(|row| row.iter().any(|v| *v == c))
        {
            2
        } else {
            0
        }
    }
}

impl NullTerminable for JisX0208 {}

#[cfg(feature = "rand")]
impl Distribution<char> for JisX0208 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        let c = rng.random_range(0..x0208_tables::RAND_MAP_0208.len() + 22);
        if c <= 21 {
            if c == 21 {
                '\x7F'
            } else {
                char::from(c as u8)
            }
        } else {
            x0208_tables::RAND_MAP_0208[c - 22]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO_WORLD_JIS0208: &[u8] = &[
        0x25, 0x4F, 0x25, 0x6D, 0x21, 0x3C, 0x25, 0x6F, 0x21, 0x3C, 0x25, 0x6B, 0x25, 0x49, 0x21,
        0x6F, 0x23, 0x6E, 0x24, 0x21,
    ];

    #[test]
    fn test_validate() {
        assert!(JisX0208::validate(HELLO_WORLD_JIS0208).is_ok());
    }

    #[test]
    fn test_decode() {
        let str = unsafe { Str::<JisX0208>::from_bytes_unchecked(HELLO_WORLD_JIS0208) };
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ハ');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ロ');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ー');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ワ');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ー');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ル');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ド');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, '¥');
        let (c, str) = JisX0208::decode_char(&str);
        assert_eq!(c, 'n');
        let (c, _) = JisX0208::decode_char(&str);
        assert_eq!(c, 'ぁ');
    }
}
