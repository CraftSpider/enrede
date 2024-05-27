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
