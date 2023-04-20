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

#[derive(Clone, Copy)]
pub enum CommunicationType {
    Exchange,
    Sync,
    AllGather,
}

#[derive(Clone, Named)]
pub struct BaseCommunicationPlugin {
    num_ranks: WorldSize,
    world_rank: WorldRank,
}

impl BaseCommunicationPlugin {
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
    sim.get_next_tag()
}

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
