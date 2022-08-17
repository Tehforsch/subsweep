use super::world_communicator::WorldCommunicator;
use super::DataByRank;
use super::Rank;
use super::SizedCommunicator;

pub struct ExchangeCommunicator<C, T> {
    communicator: C,
    data: DataByRank<Vec<T>>,
}

impl<C, T> Clone for ExchangeCommunicator<C, T>
where
    C: Clone + WorldCommunicator<T>,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            communicator: self.communicator.clone(),
            data: self.data.clone(),
        }
    }
}

impl<C, T> ExchangeCommunicator<C, T>
where
    C: WorldCommunicator<T>,
    C: SizedCommunicator,
{
    pub fn new(communicator: C) -> Self {
        let data = DataByRank::from_communicator(&communicator);
        Self {
            communicator,
            data: data,
        }
    }

    pub fn send(&mut self, rank: i32, data: T) {
        self.data.push(rank, data);
    }

    pub fn send_vec(&mut self, rank: i32, data: Vec<T>) {
        self.data[rank].extend(data)
    }

    pub fn receive_vec(&mut self) -> DataByRank<Vec<T>> {
        for (rank, data) in self.data.drain_all() {
            self.communicator.send_vec(rank, data);
        }
        let mut received_data = DataByRank::from_communicator(&self.communicator);
        for rank in self.communicator.other_ranks() {
            let moved_to_own_domain = self.communicator.receive_vec(rank);
            received_data.insert(rank, moved_to_own_domain);
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
#[cfg(feature = "local")]
mod tests {
    use std::thread;

    #[test]
    fn exchange_communicator() {
        use crate::communication::get_local_communicators;
        use crate::communication::ExchangeCommunicator;
        use crate::communication::Rank;
        use crate::communication::SizedCommunicator;
        let num_threads = 4 as i32;
        let mut communicators = get_local_communicators(num_threads as usize);
        let threads: Vec<_> = (0 as Rank..num_threads as Rank)
            .map(|rank| {
                let mut communicator =
                    ExchangeCommunicator::new(communicators.remove(&(rank as Rank)).unwrap());
                thread::spawn(move || {
                    let wrap = |x: i32| x.rem_euclid(num_threads);
                    let target_rank = wrap(rank + 1);
                    communicator.send(target_rank, rank);
                    communicator.send(target_rank, wrap(rank + 1));
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
