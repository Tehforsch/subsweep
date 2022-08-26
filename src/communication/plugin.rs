use std::marker::PhantomData;

use bevy::prelude::App;
use bevy::prelude::Plugin;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
use mpi::Tag;

use super::from_communicator::FromCommunicator;
use super::Communicator;
use super::ExchangeCommunicator;
use super::Identified;
use super::SyncCommunicator;
use super::WorldCommunicator;
use super::WorldRank;

const INITIAL_TAG: Tag = 0;

#[derive(Clone, Copy)]
pub enum CommunicationType {
    Exchange,
    Sync,
    Sum,
    AllGather,
}

pub(super) struct CurrentTag(pub(super) Tag);

pub struct CommunicationPlugin<T> {
    _marker: PhantomData<T>,
    type_: CommunicationType,
}

impl<T> CommunicationPlugin<T> {
    pub fn new(type_: CommunicationType) -> Self {
        Self {
            _marker: PhantomData::default(),
            type_,
        }
    }
}

#[cfg(not(feature = "local"))]
impl<T: Equivalence + Sync + Send + 'static> Plugin for CommunicationPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build(&self, app: &mut App) {
        let tag = match app.world.get_resource_mut::<CurrentTag>() {
            Some(mut tag) => {
                tag.0 += 1;
                tag.0
            }
            None => INITIAL_TAG,
        };
        todo!()
    }
}

#[cfg(not(feature = "local"))]
fn get_communicator<T: Equivalence>(_app: &mut App, tag: Tag) -> Communicator<T> {
    use crate::communication::mpi_world::MpiWorld;
    MpiWorld::new(tag)
}
