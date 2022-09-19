use hdf5::H5Type;

use crate::named::Named;

pub trait Attribute {
    type Output: H5Type;
    fn to_value(&self) -> Self::Output;
    fn name() -> &'static str;
}

impl<T> Named for T
where
    T: Attribute,
{
    fn name() -> &'static str {
        <T as Attribute>::name()
    }
}
