use mpi::{
    datatype::{DatatypeRef, SystemDatatype, UserDatatype},
    ffi,
    traits::{Equivalence, FromRaw},
};

use super::{dimension::Dimension, quantity::Quantity};

unsafe impl<const D: Dimension> Equivalence for Quantity<D> {
    type Out = SystemDatatype;

    fn equivalent_datatype() -> Self::Out {
        unsafe { DatatypeRef::from_raw(ffi::RSMPI_DOUBLE) }
    }
}

#[cfg(test)]
mod tests {
    use crate::units::meter;
    use mpi::traits::Communicator;

    #[test]
    fn pack_unpack_quantity() {
        let q1 = meter(1.0);
        let mut q2 = meter(2.0);

        let universe = mpi::initialize().unwrap();
        let world = universe.world();
        let a = world.pack(&q1);
        unsafe {
            world.unpack_into(&a, &mut q2, 0);
        }
    }
}
