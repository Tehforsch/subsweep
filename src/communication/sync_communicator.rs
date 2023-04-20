use bevy::prelude::Commands;
use bevy::prelude::Entity;
use mpi::traits::Equivalence;

use super::communicator::Communicator;
use super::exchange_communicator::ExchangeCommunicator;
use super::identified::EntityKey;
use super::DataByRank;
use super::Identified;
use super::Rank;
use super::SizedCommunicator;
use crate::hash_map::HashMap;
use crate::hash_map::HashSet;

pub struct SyncResult<T> {
    pub updated: DataByRank<Vec<(Entity, T)>>,
    pub deleted: DataByRank<Vec<Entity>>,
}

impl<T> SyncResult<T> {
    pub fn from_communicator(communicator: &impl SizedCommunicator) -> Self {
        Self {
            updated: DataByRank::from_communicator(communicator),
            deleted: DataByRank::from_communicator(communicator),
        }
    }

    pub fn despawn_deleted(&mut self, commands: &mut Commands) {
        for (_, entities) in self.deleted.drain_all() {
            for entity in entities.into_iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub struct SyncCommunicator<T> {
    communicator: ExchangeCommunicator<Identified<T>>,
    known: DataByRank<HashMap<EntityKey, Entity>>,
    to_sync: DataByRank<HashMap<Entity, T>>,
}

impl<T> From<Communicator<T>> for SyncCommunicator<T> {
    fn from(communicator: Communicator<T>) -> Self {
        let identified_communicator: Communicator<Identified<T>> = communicator.into();
        let known = DataByRank::from_communicator(&identified_communicator);
        let to_sync = DataByRank::from_communicator(&identified_communicator);
        Self {
            communicator: identified_communicator.into(),
            known,
            to_sync,
        }
    }
}

impl<T> SyncCommunicator<T>
where
    T: Equivalence,
{
    pub fn send_sync(&mut self, rank: Rank, entity: Entity, data: T) {
        self.to_sync[rank].insert(entity, data);
    }

    #[must_use]
    pub fn receive_sync(&mut self, mut f: impl FnMut(Rank, T) -> Entity) -> SyncResult<T> {
        let all_data: DataByRank<Vec<Identified<T>>> = self
            .to_sync
            .drain_all()
            .map(|(rank, data)| {
                (
                    rank,
                    data.into_iter()
                        .map(|(entity, data)| Identified::new(entity, data))
                        .collect(),
                )
            })
            .collect();
        let data = self.communicator.exchange_all(all_data);
        let mut result = SyncResult::from_communicator(&self.communicator);
        for (rank, data) in data.into_iter() {
            let updated = &mut result.updated[rank];
            let deleted = &mut result.deleted[rank];
            let known_this_rank = &mut self.known[rank];
            let mut known_but_not_mentioned: HashSet<_> =
                known_this_rank.iter().map(|(k, _)| *k).collect();
            for d in data.into_iter() {
                match known_this_rank.get(&d.key) {
                    Some(entity) => {
                        known_but_not_mentioned.remove(&d.key);
                        updated.push((*entity, d.data));
                    }
                    None => {
                        let new_entity = f(rank, d.data);
                        known_this_rank.insert(d.key, new_entity);
                    }
                }
            }
            for key in known_but_not_mentioned.into_iter() {
                let entity = known_this_rank.remove(&key).unwrap();
                deleted.push(entity);
            }
        }
        result
    }
}
