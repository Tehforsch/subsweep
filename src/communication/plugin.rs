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
use crate::communication::get_local_communicators;

const INITIAL_TAG: Tag = 0;

#[derive(Clone, Copy)]
pub enum CommunicationType {
    Exchange,
    Sync,
    Sum,
    AllGather,
}

// impl CommunicationType {
//     fn add_to_app<T>(&self, app: &mut App, comm: Communicator<T>) {
//         if matches!(self.type_, CommunicationType::Sync) {
//             let comm: Communicator<T> = get_communicator(app, tag);
//             app.insert_non_send_resource(SyncCommunicator::new(comm));
//         }
//         match self.type_ {
//             Self::Exchange => {
//                 app.insert_non_send_resource(ExchangeCommunicator::new(comm))
//             }
//             Self::Sum => app.insert_non_send_resource(comm),
//             Self::AllGather => app.insert_non_send_resource(comm),
//             Self::Sync => unreachable!(),
//         }
//     }
// }

struct CurrentTag(Tag);

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

#[cfg(feature = "local")]
impl<T: Equivalence + Sync + Send + 'static> Plugin for CommunicationPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build(&self, app: &mut App) {
        if !app.world.get_resource::<WorldRank>().unwrap().is_main() {
            return;
        }
        let tag = match app.world.get_resource_mut::<CurrentTag>() {
            Some(mut tag) => {
                tag.0 += 1;
                tag.0
            }
            None => INITIAL_TAG,
        };
        add_to_app::<T>(app, self.type_);
    }
}

fn add_to_app<T: 'static>(app: &mut App, type_: CommunicationType) {
    use crate::communication::NumRanks;
    use crate::SubAppRank;
    let size = *app.world.get_resource::<NumRanks>().unwrap();
    let mut comms = get_local_communicators::<T>(size.0);

    let mut add_comm = |app: &mut App, rank: i32| match type_ {
        CommunicationType::Exchange => {
            app.world
                .insert_non_send_resource(ExchangeCommunicator::from_communicator(
                    comms.remove(&rank).unwrap(),
                ))
        }
        CommunicationType::AllGather => app
            .world
            .insert_non_send_resource(comms.remove(&rank).unwrap()),
        CommunicationType::Sum => app
            .world
            .insert_non_send_resource(comms.remove(&rank).unwrap()),
        _ => unimplemented!(),
    };
    add_comm(app, 0 as i32);
    for rank in 1..size.0 {
        let subapp = app.sub_app_mut(SubAppRank::from_num(rank));
        add_comm(subapp, rank as i32)
    }
}
