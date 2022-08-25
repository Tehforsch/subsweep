use std::marker::PhantomData;

use bevy::prelude::Plugin;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
use mpi::Tag;

use super::Communicator;
use super::ExchangeCommunicator;
use super::Identified;
use super::SyncCommunicator;

const INITIAL_TAG: Tag = 0;

pub enum CommunicationType {
    Exchange,
    Sync,
    Sum,
    AllGather,
}

struct CurrentTag(Tag);

pub struct CommunicationPlugin<T> {
    _marker: PhantomData<T>,
    type_: CommunicationType,
}

impl<T: Equivalence + Sync + Send + 'static> Plugin for CommunicationPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        let tag = match app.world.get_resource_mut::<CurrentTag>() {
            Some(mut tag) => {
                tag.0 += 1;
                tag.0
            }
            None => INITIAL_TAG,
        };
        if matches!(self.type_, CommunicationType::Sync) {
            let comm: Communicator<Identified<T>> = get_communicator(app, tag);
            app.insert_non_send_resource(SyncCommunicator::new(comm));
        } else {
            let comm: Communicator<T> = get_communicator(app, tag);
            match self.type_ {
                CommunicationType::Exchange => {
                    app.insert_non_send_resource(ExchangeCommunicator::new(comm))
                }
                CommunicationType::Sum => app.insert_non_send_resource(comm),
                CommunicationType::AllGather => app.insert_non_send_resource(comm),
                CommunicationType::Sync => unreachable!(),
            };
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
}

#[cfg(feature = "local")]
fn get_communicator<T>(app: &mut bevy::prelude::App) {
    use std::sync::mpsc::channel;

    use super::ExchangeCommunicator;
    let (sender, receiver) = channel();
    app.insert_resource(ExchangeCommunicator::<T>::new());
}

#[cfg(not(feature = "local"))]
fn get_communicator<T: Equivalence>(_app: &mut bevy::prelude::App, tag: Tag) -> Communicator<T> {
    use crate::communication::mpi_world::MpiWorld;

    MpiWorld::new(tag)
}
