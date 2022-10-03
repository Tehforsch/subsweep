use std::collections::HashMap;
use std::collections::HashSet;

use bevy::prelude::Entity;
use mpi::traits::Equivalence;

use super::communicator::Communicator;
use super::exchange_communicator::ExchangeCommunicator;
use super::identified::EntityKey;
use super::DataByRank;
use super::Identified;
use super::Rank;
use super::SizedCommunicator;

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
}

pub struct SyncCommunicator<T> {
    communicator: ExchangeCommunicator<Identified<T>>,
    known: DataByRank<HashMap<EntityKey, Entity>>,
    to_sync: DataByRank<Vec<Identified<T>>>,
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
        self.to_sync[rank].push(Identified::new(entity, data));
    }

    pub fn receive_sync(&mut self, mut f: impl FnMut(Rank, T) -> Entity) -> SyncResult<T> {
        for (rank, data) in self.to_sync.drain_all() {
            self.communicator.blocking_send_vec(rank, data);
        }
        let data = self.communicator.receive_vec();
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

#[cfg(test)]
#[cfg(not(feature = "mpi"))]
pub mod tests {
    use std::thread;

    use mpi::Tag;

    use crate::communication::local::LocalCommunicator;
    use crate::communication::local_sim_building::add_receivers_to_communicator;
    use crate::communication::local_sim_building::add_senders_to_communicator;
    use crate::communication::local_sim_building::get_senders_and_receivers;
    use crate::communication::DataByRank;

    pub fn get_communicators<T>(num_threads: usize, tag: Tag) -> DataByRank<LocalCommunicator<T>> {
        let (mut senders, mut receivers) = get_senders_and_receivers(num_threads, tag);
        let mut communicators = DataByRank::empty();
        for rank in 0i32..num_threads as i32 {
            let mut communicator = LocalCommunicator::<T>::new(
                DataByRank::empty(),
                DataByRank::empty(),
                tag,
                num_threads,
                rank,
            );
            add_senders_to_communicator(&mut communicator, &mut senders);
            add_receivers_to_communicator(&mut communicator, &mut receivers);
            communicators.insert(rank, communicator);
        }
        communicators
    }

    #[test]
    #[ignore]
    fn sync_communicator() {
        use bevy::prelude::Entity;

        use super::SyncCommunicator;
        let mut communicators = get_communicators(2, 0);
        let mut communicator0 = SyncCommunicator::from(communicators.remove(&0).unwrap());
        let mut communicator1 = SyncCommunicator::from(communicators.remove(&1).unwrap());
        let entity_translation = |_, data| {
            // This makes no sense, and is just for test purposes
            Entity::from_raw(data)
        };
        let thread = thread::spawn(move || {
            // Initialize some entities
            communicator1.send_sync(0, Entity::from_raw(0), 100);
            communicator1.send_sync(0, Entity::from_raw(1), 110);
            let result = communicator1.receive_sync(entity_translation);
            // they should only be created
            assert!(result.updated[0].is_empty());
            assert!(result.deleted[0].is_empty());

            // Sync the same entities
            communicator1.send_sync(0, Entity::from_raw(0), 101);
            communicator1.send_sync(0, Entity::from_raw(1), 111);
            let result = communicator1.receive_sync(entity_translation);
            // Make sure the updated information comes in
            assert_eq!(result.updated[0][0].1, 201);
            assert_eq!(result.updated[0][1].1, 211);
            assert!(result.deleted[0].is_empty());

            // Leave out one entity on this core
            communicator1.send_sync(0, Entity::from_raw(0), 102);
            let result = communicator1.receive_sync(entity_translation);
            assert_eq!(result.updated[0][0].1, 202);
            assert_eq!(result.updated[0][1].1, 212);
            assert!(result.deleted[0].is_empty());
        });

        communicator0.send_sync(1, Entity::from_raw(0), 200);
        communicator0.send_sync(1, Entity::from_raw(1), 210);
        let result = communicator0.receive_sync(entity_translation);
        assert!(result.updated[1].is_empty());
        assert!(result.deleted[1].is_empty());

        communicator0.send_sync(1, Entity::from_raw(0), 201);
        communicator0.send_sync(1, Entity::from_raw(1), 211);
        let result = communicator0.receive_sync(entity_translation);
        assert_eq!(result.updated[1][0].1, 101);
        assert_eq!(result.updated[1][1].1, 111);
        assert!(result.deleted[1].is_empty());

        communicator0.send_sync(1, Entity::from_raw(0), 202);
        communicator0.send_sync(1, Entity::from_raw(1), 212);
        let result = communicator0.receive_sync(entity_translation);
        // Rank 1 left out one entity in the sync - make sure it is marked as deleted
        assert_eq!(result.updated[1][0].1, 102);
        assert_eq!(result.updated[1].get(1), None);
        assert_eq!(result.deleted[1][0], Entity::from_raw(110));

        thread.join().unwrap();
    }
}
