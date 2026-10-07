#[cfg(feature = "jisx0201")]
mod jisx0201;
#[cfg(feature = "jisx0208")]
mod jisx0208;
#[cfg(feature = "shiftjis")]
mod shiftjis;
#[cfg(feature = "jisx0208")]
mod x0208_tables;

#[cfg(feature = "jisx0201")]
#[doc(inline)]
pub use jisx0201::*;
#[cfg(feature = "jisx0208")]
#[doc(inline)]
pub use jisx0208::*;
#[cfg(feature = "shiftjis")]
#[doc(inline)]
pub use shiftjis::*;
