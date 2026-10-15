#[cfg(feature = "iso8859_1")]
mod iso_1;
#[cfg(feature = "iso8859_10")]
mod iso_10;
#[cfg(feature = "iso8859_11")]
mod iso_11;
#[cfg(feature = "iso8859_13")]
mod iso_13;
#[cfg(feature = "iso8859_14")]
mod iso_14;
#[cfg(feature = "iso8859_15")]
mod iso_15;
#[cfg(feature = "iso8859_16")]
mod iso_16;
#[cfg(feature = "iso8859_2")]
mod iso_2;
#[cfg(feature = "iso8859_3")]
mod iso_3;
#[cfg(feature = "iso8859_4")]
mod iso_4;
#[cfg(feature = "iso8859_5")]
mod iso_5;
#[cfg(feature = "iso8859_6")]
mod iso_6;
#[cfg(feature = "iso8859_7")]
mod iso_7;
#[cfg(feature = "iso8859_8")]
mod iso_8;
#[cfg(feature = "iso8859_9")]
mod iso_9;

#[cfg(feature = "iso8859_1")]
#[doc(inline)]
pub use iso_1::*;
#[cfg(feature = "iso8859_2")]
#[doc(inline)]
pub use iso_2::*;
#[cfg(feature = "iso8859_3")]
#[doc(inline)]
pub use iso_3::*;
#[cfg(feature = "iso8859_4")]
#[doc(inline)]
pub use iso_4::*;
#[cfg(feature = "iso8859_5")]
#[doc(inline)]
pub use iso_5::*;
#[cfg(feature = "iso8859_6")]
#[doc(inline)]
pub use iso_6::*;
#[cfg(feature = "iso8859_7")]
#[doc(inline)]
pub use iso_7::*;
#[cfg(feature = "iso8859_8")]
#[doc(inline)]
pub use iso_8::*;
#[cfg(feature = "iso8859_9")]
#[doc(inline)]
pub use iso_9::*;
#[cfg(feature = "iso8859_10")]
#[doc(inline)]
pub use iso_10::*;
#[cfg(feature = "iso8859_11")]
#[doc(inline)]
pub use iso_11::*;
#[cfg(feature = "iso8859_13")]
#[doc(inline)]
pub use iso_13::*;
#[cfg(feature = "iso8859_14")]
#[doc(inline)]
pub use iso_14::*;
#[cfg(feature = "iso8859_15")]
#[doc(inline)]
pub use iso_15::*;
#[cfg(feature = "iso8859_16")]
#[doc(inline)]
pub use iso_16::*;
