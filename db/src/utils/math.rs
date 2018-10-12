use std::cmp::Ord;

pub fn clamp<T: Ord>(i: T, min: T, max: T) -> T {
    if i < min {
        return min;
    }
    if i > max {
        return max;
    }
    i
}
