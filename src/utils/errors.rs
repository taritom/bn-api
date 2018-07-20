pub trait BigNeonError: Sized {}

pub type BigNeonResult<T> = Result<T, BigNeonError>;
