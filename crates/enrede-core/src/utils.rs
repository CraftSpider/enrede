use arrayvec::ArrayVec;
use core::ops::{Bound, RangeBounds, RangeFrom, RangeFull};

pub trait RangeOpen<T> {
    fn start_bound(&self) -> Bound<&T>;
}

impl<T> RangeOpen<T> for RangeFrom<T> {
    fn start_bound(&self) -> Bound<&T> {
        <Self as RangeBounds<T>>::start_bound(self)
    }
}

impl<T> RangeOpen<T> for RangeFull {
    fn start_bound(&self) -> Bound<&T> {
        <Self as RangeBounds<T>>::start_bound(self)
    }
}

pub trait IntoAV {
    fn into_av(self) -> ArrayVec<u8, 4>;
}

impl IntoAV for u8 {
    fn into_av(self) -> ArrayVec<u8, 4> {
        ArrayVec::from_iter([self])
    }
}

impl IntoAV for [u8; 4] {
    fn into_av(self) -> ArrayVec<u8, 4> {
        ArrayVec::from(self)
    }
}

impl IntoAV for ArrayVec<u8, 2> {
    fn into_av(self) -> ArrayVec<u8, 4> {
        ArrayVec::from_iter(self)
    }
}

impl IntoAV for ArrayVec<u8, 4> {
    fn into_av(self) -> ArrayVec<u8, 4> {
        self
    }
}
