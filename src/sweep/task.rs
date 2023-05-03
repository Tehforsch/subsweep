use mpi::datatype::UserDatatype;
use mpi::internal::memoffset::offset_of;
use mpi::traits::Equivalence;
use mpi::Address;

use super::direction::DirectionIndex;
use super::Chemistry;
use super::Flux;
use crate::particle::ParticleId;

#[derive(Debug)]
pub struct Task {
    pub id: ParticleId,
    pub dir: DirectionIndex,
}

#[derive(Clone, Debug)]
pub struct FluxData<C: Chemistry> {
    pub id: ParticleId,
    pub dir: DirectionIndex,
    pub flux: Flux<C>,
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dir.partial_cmp(&other.dir)
    }
}

impl Eq for Task {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.dir == other.dir
    }
}

unsafe impl<C: Chemistry> Equivalence for FluxData<C> {
    type Out = UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        UserDatatype::structured(
            &[1, 1, 1],
            &[
                offset_of!(FluxData<C>, id) as Address,
                offset_of!(FluxData<C>, dir) as Address,
                offset_of!(FluxData<C>, flux) as Address,
            ],
            &[
                UserDatatype::contiguous(1, &ParticleId::equivalent_datatype()),
                UserDatatype::contiguous(1, &DirectionIndex::equivalent_datatype()),
                UserDatatype::contiguous(1, &Flux::<C>::equivalent_datatype()),
            ],
        )
    }
}
