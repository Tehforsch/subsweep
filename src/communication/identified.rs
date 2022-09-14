pub(super) type EntityKey = u64;

#[derive(PartialEq, Eq)]
pub struct Identified<T> {
    pub key: EntityKey,
    pub data: T,
}

impl<T> Identified<T> {
    pub fn new(entity: Entity, data: T) -> Identified<T> {
        Self {
            key: entity.to_bits(),
            data,
        }
    }

    pub fn entity(&self) -> Entity {
        Entity::from_bits(self.key)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Identified<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identified")
            .field("key", &format_args!("{:?}", Entity::from_bits(self.key)))
            .field("data", &self.data)
            .finish()
    }
}

use mpi::datatype::UserDatatype;
use mpi::internal::memoffset::offset_of;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
use mpi::Address;

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
                offset_of!(Identified<T>, key) as Address,
                offset_of!(Identified<T>, data) as Address,
            ],
            &[
                UserDatatype::contiguous(1, &EntityKey::equivalent_datatype()),
                UserDatatype::contiguous(1, &T::equivalent_datatype()),
            ],
        )
    }
}

use bevy::prelude::Entity;

#[cfg(test)]
#[cfg(not(feature = "local"))]
mod tests {
    use bevy::prelude::Entity;
    use mpi::traits::Communicator;
    use mpi::traits::Equivalence;
    use mpi::traits::MatchesRaw;

    use super::Identified;
    use crate::communication::MPI_UNIVERSE;
    use crate::units::VecLength;

    #[derive(Clone, Default, Equivalence, PartialEq, Eq, Debug)]
    struct ComplexStruct {
        i: [i32; 3],
        b: bool,
    }

    #[derive(Clone, Default, Debug, Equivalence, PartialEq)]
    struct A {
        pos: VecLength,
    }

    fn test_pack_unpack<T>(data: T)
    where
        T: Clone + Default + Equivalence + core::fmt::Debug + PartialEq,
        <T as Equivalence>::Out: MatchesRaw,
    {
        let world = MPI_UNIVERSE.world();

        for num in [0, 50, 100, 1000000].iter() {
            for generation in [0, 50, 100, 1000000].iter() {
                let q1 = Identified::new(
                    Entity::from_bits((u32::MAX as u64) * generation + 1 + *num),
                    data.clone(),
                );
                let mut q2 = Identified::new(Entity::from_raw(0), T::default());
                let a = world.pack(&q1);
                unsafe {
                    world.unpack_into(&a, &mut q2, 0);
                }
                assert_eq!(q1, q2);
            }
        }
    }

    #[test]
    fn pack_unpack_identified() {
        test_pack_unpack(ComplexStruct {
            i: [15, 19, 50],
            b: false,
        });
        test_pack_unpack(A {
            pos: VecLength::zero(),
        })
    }
}
