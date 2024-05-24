use crate::encoding::Encoding;
use crate::str::Str;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::slice;

/// Character iterator for encoded strings. This iterates the encoding yielding Unicode code points.
pub struct Chars<'a, E> {
    str: &'a Str<E>,
    _phantom: PhantomData<E>,
}

impl<'a, E: Encoding> Chars<'a, E> {
    pub(super) fn new(str: &'a Str<E>) -> Self {
        Chars {
            str,
            _phantom: PhantomData,
        }
    }
}

impl<'a, E: Encoding> Iterator for Chars<'a, E> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.str.is_empty() {
            return None;
        }
        let (c, str) = E::decode_char(self.str);
        self.str = str;
        Some(c)
    }
}

impl<'a, E: Encoding> FusedIterator for Chars<'a, E> where slice::Iter<'a, u8>: FusedIterator {}

/// Character and index iterator for encoded strings. This iterates the encoding yielding Unicode
/// code points and their byte index in the encoded string.
pub struct CharIndices<'a, E> {
    offset: usize,
    iter: Chars<'a, E>,
}

impl<'a, E: Encoding> CharIndices<'a, E> {
    pub(super) fn new(str: &'a Str<E>) -> Self {
        CharIndices {
            offset: 0,
            iter: Chars::new(str),
        }
    }
}

impl<'a, E: Encoding> Iterator for CharIndices<'a, E> {
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

impl<'a, E: Encoding> FusedIterator for CharIndices<'a, E> where Chars<'a, E>: FusedIterator {}
