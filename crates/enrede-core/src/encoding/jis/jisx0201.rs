use crate::encoding::sealed::Sealed;
use crate::encoding::{Enc, NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

pub(crate) const DECODE_MAP_0201: [char; 63] = [
    '｡', '｢', '｣', '､', '･', 'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｰ', 'ｱ', 'ｲ', 'ｳ',
    'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ',
    'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ',
    'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ', 'ﾞ', 'ﾟ',
];

/// The [JIS X 0201](https://en.wikipedia.org/wiki/JIS_X_0201) encoding.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct JisX0201;

impl Sealed for JisX0201 {}

impl Encoding for JisX0201 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "jisx0201"
    }

    fn dyn_enc() -> Enc {
        Enc::JisX0201
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x20).contains(c) || (0x80..0xA1).contains(c) || (0xE0..).contains(c) {
                Err(ValidateError {
                    valid_up_to: idx,
                    error_len: Some(1),
                })
            } else {
                Ok(())
            }
        })
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if c == '¥' {
            Some(0x5C)
        } else if c == '‾' {
            Some(0x7E)
        } else if (0x20..0x80).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_0201.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA1)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if b == 0x5C {
            ('¥', &str[1..])
        } else if b == 0x7E {
            ('‾', &str[1..])
        } else if (..0x80).contains(&b) {
            (b as char, &str[1..])
        } else {
            (DECODE_MAP_0201[b as usize - 0xA1], &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (0x20..0x80).contains(&(c as u32)) || DECODE_MAP_0201.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for JisX0201 {}

#[cfg(feature = "rand")]
impl Distribution<char> for JisX0201 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Number of JIS 0201 characters
        let c = rng.random_range(0..159);
        let c = if c < 0x60 { c + 0x20 } else { c + 0x41 };
        Self::decode_char(unsafe { Str::from_bytes_unchecked(&[c]) }).0
    }
}
