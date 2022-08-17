pub(super) type EntityKey = u64;

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
