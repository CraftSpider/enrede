use crate::encoding::jis::x0208_tables;
use crate::encoding::sealed::Sealed;
use crate::encoding::{Enc, JisX0201, NullTerminable, ValidateError, DECODE_MAP_0201};
use crate::{Encoding, Str};
use arrayvec::ArrayVec;
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

/// The [ShiftJIS](https://en.wikipedia.org/wiki/Shift_JIS) encoding.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ShiftJIS;

impl Sealed for ShiftJIS {}

impl Encoding for ShiftJIS {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 2;
    type Bytes = ArrayVec<u8, 2>;

    fn shorthand() -> &'static str {
        "shiftjis"
    }

    fn dyn_enc() -> Enc {
        Enc::ShiftJIS
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        let mut row = 0;
        for (idx, b) in bytes.iter().enumerate() {
            if row == 0 {
                // Single-byte characters
                if (0..0x80).contains(b) || (0xA1..0xE0).contains(b) {
                    continue;
                // First set of row values
                } else if (0x81..0xA0).contains(b) {
                    row = *b - 0x80;
                // Second set of row values
                } else if (0xE0..0xF0).contains(b) {
                    row = *b - 0xC1;
                } else {
                    return Err(ValidateError {
                        valid_up_to: idx,
                        error_len: Some(1),
                    });
                }
            } else if row != 0 {
                row -= 1;
                let column = if (0x9F..0xFD).contains(b) {
                    row = row * 2 + 1;
                    *b - 0x9F
                } else if (0x40..0x7F).contains(b) {
                    row *= 2;
                    *b - 0x40
                } else if (0x80..0x9F).contains(b) {
                    row *= 2;
                    *b - 0x41
                } else {
                    return Err(ValidateError {
                        valid_up_to: idx - 1,
                        error_len: Some(2),
                    });
                };
                if x0208_tables::DECODE_MAP_0208[row as usize][column as usize] == '�' {
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
        match JisX0201::encode_char(c) {
            Some(c) => return Some(ArrayVec::from_iter([c])),
            None => (),
        }
        let (row, col) = x0208_tables::ENCODE_MAP_0208[&c];
        let row = row + 0x21;
        let row_e = if row <= 0x5E {
            ((row + 1) / 2) + 112
        } else {
            ((row + 1) / 2) + 176
        };
        let col_e = if row % 2 == 0 {
            col + 159
        } else {
            col + 64 + (col / 63)
        };
        Some(ArrayVec::from([row_e as u8, col_e as u8]))
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let first = bytes[0];
        if (..0x80).contains(&first) || (0xA1..0xE0).contains(&first) {
            let c = if first == 0x5C {
                '¥'
            } else if first == 0x7E {
                '‾'
            } else if (..0x80).contains(&first) {
                first as char
            } else {
                DECODE_MAP_0201[first as usize - 0xA1]
            };
            (c, unsafe { str.get_unchecked(1..) })
        } else {
            let second = bytes[1];
            let mut row = if (0x81..0xA0).contains(&first) {
                first - 0x81
            } else {
                first - 0xC1
            };
            let col = if (0x40..0x7F).contains(&second) {
                row *= 2;
                second - 0x40
            } else if (0x80..0x9F).contains(&second) {
                row *= 2;
                second - 0x41
            // (0x9F..0xFD).contains(&second)
            } else {
                row = row * 2 + 1;
                second - 0x9F
            };
            let c = x0208_tables::DECODE_MAP_0208[row as usize][col as usize];
            (c, unsafe { str.get_unchecked(2..) })
        }
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        let bytes = str.as_bytes();
        let first = bytes[0];
        // Control code bytes, space, and del - always single-byte, never used as a second byte
        if (..0x40).contains(&first) || first == 0x7F {
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
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_0201.contains(&c) {
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

impl NullTerminable for ShiftJIS {}

#[cfg(feature = "rand")]
impl Distribution<char> for ShiftJIS {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        let c = rng.random_range(0..(x0208_tables::RAND_MAP_0208.len() + 158));
        if c <= 158 {
            let c = if c < 0x60 { c + 0x20 } else { c + 0x41 };
            JisX0201::decode_char(unsafe { Str::from_bytes_unchecked(&[c as u8]) }).0
        } else {
            x0208_tables::RAND_MAP_0208[c - 159]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO_WORLD_SHIFTJIS: &[u8] = &[
        0x83, 0x6E, 0x83, 0x8D, 0x81, 0x5B, 0x83, 0x8F, 0x81, 0x5B, 0x83, 0x8B, 0x83, 0x68, 0x5C,
        0x6E, 0x82, 0x9F,
    ];

    #[test]
    fn test_validate() {
        assert!(ShiftJIS::validate(HELLO_WORLD_SHIFTJIS).is_ok());
    }

    #[test]
    fn test_encode() {
        assert_eq!(
            ShiftJIS::encode_char('ハ'),
            Some(ArrayVec::from_iter([0x83, 0x6E]))
        );
        assert_eq!(
            ShiftJIS::encode_char('ロ'),
            Some(ArrayVec::from_iter([0x83, 0x8D]))
        );
        assert_eq!(
            ShiftJIS::encode_char('ー'),
            Some(ArrayVec::from_iter([0x81, 0x5B]))
        );
        assert_eq!(
            ShiftJIS::encode_char('ワ'),
            Some(ArrayVec::from_iter([0x83, 0x8F]))
        );
        assert_eq!(
            ShiftJIS::encode_char('ル'),
            Some(ArrayVec::from_iter([0x83, 0x8B]))
        );
        assert_eq!(
            ShiftJIS::encode_char('ド'),
            Some(ArrayVec::from_iter([0x83, 0x68]))
        );
        assert_eq!(
            ShiftJIS::encode_char('¥'),
            Some(ArrayVec::from_iter([0x5C]))
        );
        assert_eq!(
            ShiftJIS::encode_char('n'),
            Some(ArrayVec::from_iter([0x6E]))
        );
        assert_eq!(
            ShiftJIS::encode_char('ぁ'),
            Some(ArrayVec::from_iter([0x82, 0x9F]))
        );
    }

    #[test]
    fn test_decode() {
        let str = unsafe { Str::<ShiftJIS>::from_bytes_unchecked(HELLO_WORLD_SHIFTJIS) };
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ハ');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ロ');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ー');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ワ');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ー');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ル');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ド');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, '¥');
        let (c, str) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'n');
        let (c, _) = ShiftJIS::decode_char(&str);
        assert_eq!(c, 'ぁ');
    }
}
