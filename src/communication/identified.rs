pub(super) type EntityKey = u64;

#[derive(Debug, PartialEq, Eq)]
pub struct Identified<T> {
    pub key: EntityKey,
    pub data: T,
}

#[cfg(not(feature = "local"))]
#[path = ""]
mod identified_mpi_impl {
    use mpi::datatype::UserDatatype;
    use mpi::internal::memoffset::offset_of;
    use mpi::traits::Equivalence;
    use mpi::traits::MatchesRaw;
    use mpi::Address;

    use super::EntityKey;
    use super::Identified;

    unsafe impl<'a, T> Equivalence for Identified<T>
    where
        T: Equivalence,
        <T as Equivalence>::Out: MatchesRaw,
    {
        type Out = UserDatatype;

        fn equivalent_datatype() -> Self::Out {
            UserDatatype::structured(
                &[1, 1],
                &[
                    offset_of!(Identified<i32>, key) as Address,
                    offset_of!(Identified<i32>, data) as Address,
                ],
                &[
                    UserDatatype::contiguous(1, &EntityKey::equivalent_datatype()),
                    UserDatatype::contiguous(1, &T::equivalent_datatype()),
                ],
            )
        }
    }
}

#[cfg(not(feature = "local"))]
pub use identified_mpi_impl::*;

#[cfg(test)]
#[cfg(not(feature = "local"))]
mod tests {
    use mpi::traits::Communicator;
    use mpi::traits::Equivalence;

    use super::Identified;
    use crate::communication::MPI_UNIVERSE;

    #[derive(Equivalence, PartialEq, Eq, Debug)]
    struct ComplexStruct {
        i: [i32; 3],
        b: bool,
    }

    #[test]
    fn pack_unpack_identified() {
        let world = MPI_UNIVERSE.world();

        let q1 = Identified {
            key: 0,
            data: ComplexStruct {
                i: [1, 2, 3],
                b: false,
            },
        };
        let mut q2 = Identified {
            key: 0,
            data: ComplexStruct {
                i: [4, 5, 6],
                b: true,
            },
        };
        let a = world.pack(&q1);
        unsafe {
            world.unpack_into(&a, &mut q2, 0);
        }
        assert_eq!(q1, q2);
    }
}
