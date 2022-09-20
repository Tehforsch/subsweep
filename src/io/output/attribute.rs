use hdf5::H5Type;

use crate::named::Named;

pub trait Attribute: Named {
    type Output: H5Type;
    fn to_value(&self) -> Self::Output;
}
