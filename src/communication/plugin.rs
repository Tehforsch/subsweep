use std::marker::PhantomData;

use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
use mpi::Tag;

use super::communicator::Communicator;
use super::exchange_communicator::ExchangeCommunicator;
use super::sync_communicator::SyncCommunicator;
use super::WorldRank;
use super::WorldSize;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

pub(super) const INITIAL_TAG: Tag = 0;

#[derive(Clone, Copy)]
pub enum CommunicationType {
    Exchange,
    Sync,
    AllGather,
}

pub(super) struct CurrentTag(pub(super) Tag);

#[derive(Clone, Named)]
pub struct BaseCommunicationPlugin {
    num_ranks: WorldSize,
    world_rank: WorldRank,
}

impl BaseCommunicationPlugin {
    #[cfg(any(feature = "mpi", test))]
    pub fn new(size: usize, rank: super::Rank) -> Self {
        Self {
            num_ranks: WorldSize(size),
            world_rank: WorldRank(rank),
        }
    }
}

impl RaxiomPlugin for BaseCommunicationPlugin {
    fn build_once_everywhere(&self, sim: &mut Simulation) {
        sim.insert_resource(self.world_rank)
            .insert_resource(self.num_ranks);
    }
}

#[derive(Named)]
pub struct CommunicationPlugin<T> {
    _marker: PhantomData<T>,
    pub(super) type_: CommunicationType,
}

impl<T> Default for CommunicationPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
            type_: CommunicationType::AllGather,
        }
    }
}

impl<T> CommunicationPlugin<T> {
    pub fn new(type_: CommunicationType) -> Self {
        Self {
            _marker: PhantomData::default(),
            type_,
        }
    }

    pub fn sync() -> Self {
        Self::new(CommunicationType::Sync)
    }

    pub fn exchange() -> Self {
        Self::new(CommunicationType::Exchange)
    }
}

pub(super) fn get_next_tag(sim: &mut Simulation) -> Tag {
    let mut tag = sim
        .get_resource_mut::<CurrentTag>()
        .map(|x| x.0)
        .unwrap_or(INITIAL_TAG);
    tag += 1;
    sim.insert_resource(CurrentTag(tag));
    tag
}

#[cfg(feature = "mpi")]
impl<T: Equivalence + Sync + Send + 'static> RaxiomPlugin for CommunicationPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build_everywhere(&self, sim: &mut Simulation) {
        let tag = get_next_tag(sim);
        super::verify_tag_type_mapping::verify_tag_type_mapping::<T>(tag);
        add_communicator(self.type_, sim, Communicator::<T>::new(tag));
    }

    fn allow_adding_twice(&self) -> bool {
        true
    }
}

pub(super) fn add_communicator<T: Equivalence + 'static + Sync + Send>(
    type_: CommunicationType,
    sim: &mut Simulation,
    communicator: Communicator<T>,
) where
    <T as Equivalence>::Out: MatchesRaw,
{
    match type_ {
        CommunicationType::Exchange => {
            sim.insert_non_send_resource(ExchangeCommunicator::from(communicator));
        }
        CommunicationType::Sync => {
            sim.insert_non_send_resource(SyncCommunicator::from(communicator));
        }
        CommunicationType::AllGather => {
            sim.insert_non_send_resource(communicator);
        }
    }
}
