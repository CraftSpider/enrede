#[cfg(feature = "jisx0201")]
mod jisx0201;
#[cfg(feature = "jisx0208")]
mod jisx0208;
#[cfg(feature = "shiftjis")]
mod shiftjis;
#[cfg(feature = "jisx0208")]
mod x0208_tables;

#[cfg(feature = "jisx0201")]
pub use jisx0201::*;
#[cfg(feature = "jisx0208")]
pub use jisx0208::*;
#[cfg(feature = "shiftjis")]
pub use shiftjis::*;
