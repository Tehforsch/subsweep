use std::marker::PhantomData;

use bevy::prelude::App;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
use mpi::Tag;

use super::from_communicator::FromCommunicator;
use super::Communicator;
use super::ExchangeCommunicator;
use super::Rank;
use super::SyncCommunicator;
use super::WorldRank;
use super::WorldSize;
use crate::named::Named;
use crate::plugin_utils::Simulation;
use crate::plugin_utils::TenetPlugin;

pub(super) const INITIAL_TAG: Tag = 0;

#[derive(Clone, Copy)]
pub enum CommunicationType {
    Exchange,
    Sync,
    Sum,
    AllGather,
}

pub(super) struct CurrentTag(pub(super) Tag);

pub struct BaseCommunicationPlugin {
    num_ranks: WorldSize,
    world_rank: WorldRank,
}

impl BaseCommunicationPlugin {
    pub fn new(size: usize, rank: Rank) -> Self {
        Self {
            num_ranks: WorldSize(size),
            world_rank: WorldRank(rank),
        }
    }
}

impl Named for BaseCommunicationPlugin {
    fn name() -> &'static str {
        "base_communication"
    }
}

impl TenetPlugin for BaseCommunicationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.insert_resource(self.world_rank)
            .insert_resource(self.num_ranks);
    }

    // We can't check the world rank for this plugin
    // because it has not been initialized yet when
    // run_once runs
    fn skip_running_once(&self) -> bool {
        true
    }
}

pub struct CommunicationPlugin<T> {
    _marker: PhantomData<T>,
    pub(super) type_: CommunicationType,
}

impl<T> CommunicationPlugin<T> {
    pub fn new(type_: CommunicationType) -> Self {
        Self {
            _marker: PhantomData::default(),
            type_,
        }
    }
}

pub(super) fn get_next_tag(app: &mut App) -> Tag {
    let mut tag = app
        .world
        .get_resource_mut::<CurrentTag>()
        .map(|x| x.0)
        .unwrap_or(INITIAL_TAG);
    tag += 1;
    app.world.insert_resource(CurrentTag(tag));
    tag
}

#[cfg(feature = "mpi")]
impl<T: Equivalence + Sync + Send + 'static> bevy::prelude::Plugin for CommunicationPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build(&self, app: &mut App) {
        let tag = get_next_tag(app);
        add_communicator(self.type_, app, Communicator::<T>::new(tag));
    }
}

pub(super) fn add_communicator<T: Equivalence + 'static + Sync + Send>(
    type_: CommunicationType,
    app: &mut App,
    communicator: Communicator<T>,
) where
    <T as Equivalence>::Out: MatchesRaw,
{
    match type_ {
        CommunicationType::Exchange => {
            app.insert_non_send_resource(ExchangeCommunicator::from_communicator(communicator));
        }
        CommunicationType::Sync => {
            app.insert_non_send_resource(SyncCommunicator::from_communicator(communicator.into()));
        }
        CommunicationType::Sum => {
            app.insert_non_send_resource(communicator);
        }
        CommunicationType::AllGather => {
            app.insert_non_send_resource(communicator);
        }
    }
}
