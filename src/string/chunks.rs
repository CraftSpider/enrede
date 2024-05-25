use crate::encoding::Encoding;
use crate::str::Str;
use core::marker::PhantomData;

pub(super) struct EncodedChunks<'a, E> {
    src: &'a [u8],
    _phantom: PhantomData<E>,
}

impl<'a, E: Encoding> EncodedChunks<'a, E> {
    pub(super) fn new(src: &'a [u8]) -> Self {
        EncodedChunks {
            src,
            _phantom: PhantomData,
        }
    }
}

pub(crate) struct EncodedChunk<'a, E> {
    valid: &'a Str<E>,
    invalid: &'a [u8],
    _phantom: PhantomData<E>,
}

impl<'a, E: Encoding> EncodedChunk<'a, E> {
    pub(super) fn valid(&self) -> &'a Str<E> {
        self.valid
    }

    pub(super) fn invalid(&self) -> &'a [u8] {
        self.invalid
    }
}

impl<'a, E: Encoding + 'a> Iterator for EncodedChunks<'a, E> {
    type Item = EncodedChunk<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            return None;
        }

        Some(match Str::<E>::from_bytes(self.src) {
            Ok(valid) => {
                let out = EncodedChunk {
                    valid,
                    invalid: &[],
                    _phantom: PhantomData,
                };
                self.src = &[];
                out
            }
            Err(err) => {
                let valid_to = err.valid_up_to();
                // SAFETY: Data up to `valid_to` is guaranteed valid for the provided encoding
                let valid = unsafe { Str::from_bytes_unchecked(&self.src[..valid_to]) };
                let invalid = match err.error_len() {
                    Some(len) => {
                        let i = &self.src[valid_to..valid_to + len];
                        self.src = &self.src[valid_to + len..];
                        i
                    }
                    None => {
                        let i = &self.src[valid_to..];
                        self.src = &[];
                        i
                    }
                };

                EncodedChunk {
                    valid,
                    invalid,
                    _phantom: PhantomData,
                }
            }
        })
    }
}
