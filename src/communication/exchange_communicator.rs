use std::marker::PhantomData;

use mpi::request::scope;

use super::from_communicator::FromCommunicator;
use super::world_communicator::WorldCommunicator;
use super::DataByRank;
use super::Rank;
use super::SizedCommunicator;

#[derive(Clone)]
pub struct ExchangeCommunicator<C, T> {
    pub communicator: C,
    pending_data: DataByRank<bool>,
    _marker: PhantomData<T>,
}

impl<C, T> FromCommunicator<C> for ExchangeCommunicator<C, T>
where
    C: SizedCommunicator,
{
    fn from_communicator(communicator: C) -> Self {
        let pending_data = DataByRank::from_communicator(&communicator);
        Self {
            communicator,
            pending_data,
            _marker: PhantomData::default(),
        }
    }
}

impl<C, T> ExchangeCommunicator<C, T>
where
    C: WorldCommunicator<T>,
    C: SizedCommunicator,
{
    pub fn send(&mut self, rank: i32, data: T) {
        self.blocking_send_vec(rank, vec![data]);
    }

    pub fn receive(&mut self) -> DataByRank<T> {
        let data = self.receive_vec();
        data.into_iter()
            .map(|(rank, mut data)| {
                debug_assert_eq!(data.len(), 1);
                (rank, data.remove(0))
            })
            .collect()
    }

    pub fn blocking_send_vec(&mut self, rank: i32, data: Vec<T>) {
        debug_assert!(!self.pending_data[rank]);
        self.pending_data[rank] = true;
        self.communicator.blocking_send_vec(rank, &data);
    }

    pub fn empty_send_to_others(&mut self) {
        for rank in self.communicator.other_ranks() {
            if !self.pending_data[rank] {
                self.blocking_send_vec(rank, vec![]);
            }
        }
    }

    pub fn exchange_all(&mut self, data: DataByRank<Vec<T>>) -> DataByRank<Vec<T>> {
        scope(|scope| {
            let mut guards = vec![];
            for (rank, items) in data.iter() {
                debug_assert!(!self.pending_data[*rank]);
                self.pending_data[*rank] = true;
                let guard = self.communicator.immediate_send_vec(scope, *rank, items);
                guards.extend(guard.into_iter());
            }
            self.receive_vec()
        })
    }

    pub fn receive_vec(&mut self) -> DataByRank<Vec<T>> {
        self.empty_send_to_others();
        let mut received_data = DataByRank::from_communicator(&self.communicator);
        for rank in self.communicator.other_ranks() {
            debug_assert!(!self.pending_data[rank]);
        }
        for rank in self.communicator.other_ranks() {
            let received = self.communicator.receive_vec(rank);
            received_data.insert(rank, received);
            self.pending_data[rank] = false;
        }
        received_data
    }
}

impl<C, T> SizedCommunicator for ExchangeCommunicator<C, T>
where
    C: SizedCommunicator,
{
    fn rank(&self) -> Rank {
        self.communicator.rank()
    }

    fn size(&self) -> usize {
        self.communicator.size()
    }
}

#[cfg(test)]
#[cfg(not(feature = "mpi"))]
mod tests {
    use std::thread;

    use crate::communication::from_communicator::FromCommunicator;
    use crate::communication::sync_communicator::tests::get_communicators;
    use crate::communication::SizedCommunicator;

    #[test]
    fn exchange_communicator() {
        use crate::communication::ExchangeCommunicator;
        use crate::communication::Rank;
        let num_threads = 4 as i32;
        let tag = 0;
        let mut communicators = get_communicators(num_threads as usize, tag);
        let threads: Vec<_> = (0 as Rank..num_threads as Rank)
            .map(|rank| {
                let mut communicator = ExchangeCommunicator::from_communicator(
                    communicators.remove(&(rank as Rank)).unwrap(),
                );
                thread::spawn(move || {
                    let wrap = |x: i32| x.rem_euclid(num_threads);
                    let target_rank = wrap(rank + 1);
                    communicator.blocking_send_vec(target_rank, vec![rank, wrap(rank + 1)]);
                    let received = communicator.receive_vec();
                    for other_rank in communicator.other_ranks() {
                        if other_rank == wrap(rank - 1) {
                            assert_eq!(
                                received.get(&other_rank).unwrap(),
                                &vec![wrap(rank - 1), rank]
                            );
                        } else {
                            assert_eq!(received.get(&other_rank).unwrap(), &Vec::<i32>::new());
                        }
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
}
