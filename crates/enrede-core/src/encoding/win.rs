#[cfg(feature = "win1251")]
mod win_1251;
#[cfg(feature = "win1252")]
mod win_1252;

#[cfg(feature = "win1251")]
pub use win_1251::*;
#[cfg(feature = "win1252")]
pub use win_1252::*;
