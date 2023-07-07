use std::path::Path;

use hdf5::File;

use crate::io::output::ToAttribute;

pub trait FromAttribute: ToAttribute {
    fn from_value(val: <Self as ToAttribute>::Output) -> Self;
}

pub fn read_attribute<T: FromAttribute>(file: &Path) -> T {
    let f = File::open(file).unwrap();
    T::from_value(f.attr(T::name()).unwrap().read_scalar().unwrap())
}
