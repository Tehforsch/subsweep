use mpi::datatype::DatatypeRef;
use mpi::datatype::SystemDatatype;
use mpi::ffi;
use mpi::traits::Equivalence;
use mpi::traits::FromRaw;

use super::dimension::Dimension;
use super::quantity::Quantity;

unsafe impl<const D: Dimension> Equivalence for Quantity<D> {
    type Out = SystemDatatype;

    fn equivalent_datatype() -> Self::Out {
        unsafe { DatatypeRef::from_raw(ffi::RSMPI_DOUBLE) }
    }
}

#[cfg(test)]
mod tests {
    use mpi::traits::Communicator;

    use crate::units::meter;

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
