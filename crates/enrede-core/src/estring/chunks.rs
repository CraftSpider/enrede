use crate::encoding::Enc;
use crate::estr::EncStr;

pub(super) struct EncodedChunks<'a> {
    enc: Enc,
    src: &'a [u8],
}

impl<'a> EncodedChunks<'a> {
    pub(super) fn new(enc: Enc, src: &'a [u8]) -> Self {
        EncodedChunks { enc, src }
    }
}

pub(crate) struct EncodedChunk<'a> {
    valid: &'a EncStr,
    invalid: &'a [u8],
}

impl<'a> EncodedChunk<'a> {
    pub(super) fn valid(&self) -> &'a EncStr {
        self.valid
    }

    pub(super) fn invalid(&self) -> &'a [u8] {
        self.invalid
    }
}

impl<'a> Iterator for EncodedChunks<'a> {
    type Item = EncodedChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            return None;
        }

        Some(match EncStr::from_bytes(self.enc, self.src) {
            Ok(valid) => {
                let out = EncodedChunk {
                    valid,
                    invalid: &[],
                };
                self.src = &[];
                out
            }
            Err(err) => {
                let valid_to = err.valid_up_to();
                // SAFETY: Data up to `valid_to` is guaranteed valid for the provided encoding
                let valid =
                    unsafe { EncStr::from_bytes_unchecked(self.enc, &self.src[..valid_to]) };
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

                EncodedChunk { valid, invalid }
            }
        })
    }
}
