use std::iter::Sum;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::collective_communicator::SumCommunicator;
use super::sized_communicator::SizedCommunicator;
use super::world_communicator::WorldCommunicator;
use super::CollectiveCommunicator;
use super::DataByRank;
use super::Rank;

pub struct LocalCommunicator<T> {
    senders: DataByRank<Sender<Vec<T>>>,
    receivers: DataByRank<Receiver<Vec<T>>>,
    rank: Rank,
    size: usize,
}

impl<T> LocalCommunicator<T> {
    pub fn new(
        rank: Rank,
        size: usize,
        senders: DataByRank<Sender<Vec<T>>>,
        receivers: DataByRank<Receiver<Vec<T>>>,
    ) -> Self {
        Self {
            senders,
            receivers,
            rank,
            size,
        }
    }
}

pub fn get_local_communicators<T>(num_threads: usize) -> DataByRank<LocalCommunicator<T>> {
    let mut senders_and_receivers: Vec<Vec<_>> = (0..num_threads)
        .map(|_| {
            (0..num_threads)
                .map(|_| {
                    let (sender, receiver) = channel();
                    (Some(sender), Some(receiver))
                })
                .collect()
        })
        .collect();
    let mut communicators = DataByRank::empty();
    for rank in 0..num_threads {
        let mut senders = DataByRank::empty();
        let mut receivers = DataByRank::empty();
        for rank2 in 0..num_threads {
            if rank == rank2 {
                continue;
            }
            senders.insert(
                rank2 as Rank,
                senders_and_receivers[rank][rank2].0.take().unwrap(),
            );
            receivers.insert(
                rank2 as Rank,
                senders_and_receivers[rank2][rank].1.take().unwrap(),
            );
        }
        communicators.insert(
            rank as Rank,
            LocalCommunicator::new(rank as Rank, num_threads, senders, receivers),
        );
    }
    communicators
}

impl<T> WorldCommunicator<T> for LocalCommunicator<T> {
    fn receive_vec(&mut self, rank: Rank) -> Vec<T> {
        let result = self.receivers[rank].recv().unwrap();
        result
    }

    fn send_vec(&mut self, rank: Rank, data: Vec<T>) {
        self.senders.get(&rank).unwrap().send(data).unwrap();
    }
}

impl<T> SizedCommunicator for LocalCommunicator<T> {
    fn rank(&self) -> Rank {
        self.rank
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl<T: Clone> CollectiveCommunicator<T> for LocalCommunicator<T> {
    fn all_gather(&mut self, data: &T) -> Vec<T> {
        for rank in self.other_ranks() {
            self.send_vec(rank, vec![data.clone()]);
        }
        let mut result = vec![];
        for rank in self.all_ranks() {
            if rank == self.rank {
                result.push(data.clone());
            } else {
                let received = self.receive_vec(rank);
                assert_eq!(received.len(), 1);
                result.extend(received.into_iter());
            }
        }
        result
    }
}

impl<T: Sum + Clone> SumCommunicator<T> for LocalCommunicator<T> {
    fn collective_sum(&mut self, send: &T) -> T {
        // We don't care about efficiency in the local communicator
        let result = self.all_gather(send);
        result.into_iter().sum()
    }
}
