use hdf5::H5Type;

pub trait Attribute {
    type Output: H5Type;
    fn to_value(&self) -> Self::Output;
}
