use crate::encoding::Enc;
use crate::estr::EncStr;
use core::iter::FusedIterator;
use core::slice;

/// Character iterator for encoded strings. This iterates the encoding yielding Unicode code points.
pub struct Chars<'a> {
    str: EncStr<'a>,
}

impl<'a> Chars<'a> {
    pub(super) fn new(str: EncStr<'a>) -> Self {
        Chars { str }
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.str.is_empty() {
            return None;
        }
        let (c, str) = unsafe { Enc::decode_char_unchecked(self.str.encoding, self.str) };
        self.str = str;
        Some(c)
    }
}

impl<'a> FusedIterator for Chars<'a> where slice::Iter<'a, u8>: FusedIterator {}

/// Character and index iterator for encoded strings. This iterates the encoding yielding Unicode
/// code points and their byte index in the encoded string.
pub struct CharIndices<'a> {
    offset: usize,
    iter: Chars<'a>,
}

impl<'a> CharIndices<'a> {
    pub(super) fn new(str: EncStr<'a>) -> Self {
        CharIndices {
            offset: 0,
            iter: Chars::new(str),
        }
    }
}

impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let pre_len = self.iter.str.len();
        let c = self.iter.next()?;
        let offset = self.offset;
        let len = self.iter.str.len();
        self.offset += pre_len - len;
        Some((offset, c))
    }
}

impl<'a> FusedIterator for CharIndices<'a> where Chars<'a>: FusedIterator {}
