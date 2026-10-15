#[cfg(feature = "win1251")]
mod win_1251;
#[cfg(feature = "win1252")]
mod win_1252;

#[cfg(feature = "win1251")]
#[doc(inline)]
pub use win_1251::*;
#[cfg(feature = "win1252")]
#[doc(inline)]
pub use win_1252::*;
