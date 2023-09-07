use mpi::datatype::UserDatatype;
use mpi::internal::memoffset::offset_of;
use mpi::traits::Equivalence;
use mpi::Address;

use super::direction::DirectionIndex;
use super::Chemistry;
use super::Rate;
use crate::particle::ParticleId;

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    pub id: ParticleId,
    pub dir: DirectionIndex,
}

#[derive(Clone, Debug)]
pub struct RateData<C: Chemistry> {
    pub id: ParticleId,
    pub dir: DirectionIndex,
    pub rate: Rate<C>,
    pub periodic: bool,
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dir.partial_cmp(&other.dir)
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

unsafe impl<C: Chemistry> Equivalence for RateData<C> {
    type Out = UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        UserDatatype::structured(
            &[1, 1, 1, 1],
            &[
                offset_of!(RateData<C>, id) as Address,
                offset_of!(RateData<C>, dir) as Address,
                offset_of!(RateData<C>, rate) as Address,
                offset_of!(RateData<C>, periodic) as Address,
            ],
            &[
                UserDatatype::contiguous(1, &ParticleId::equivalent_datatype()),
                UserDatatype::contiguous(1, &DirectionIndex::equivalent_datatype()),
                UserDatatype::contiguous(1, &Rate::<C>::equivalent_datatype()),
                UserDatatype::contiguous(1, &bool::equivalent_datatype()),
            ],
        )
    }
}
