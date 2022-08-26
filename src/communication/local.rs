use std::iter::Sum;
use std::marker::PhantomData;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use mpi::Tag;

use super::collective_communicator::SumCommunicator;
use super::sized_communicator::SizedCommunicator;
use super::world_communicator::WorldCommunicator;
use super::CollectiveCommunicator;
use super::DataByRank;
use super::Rank;

pub(super) struct Payload;

pub struct LocalCommunicator<T> {
    pub(super) senders: DataByRank<Sender<Payload>>,
    pub(super) receivers: DataByRank<Receiver<Payload>>,
    rank: Rank,
    size: usize,
    marker_: PhantomData<T>,
    _tag: Tag,
}

impl<T> LocalCommunicator<T> {
    pub(super) fn new(
        receivers: DataByRank<Receiver<Payload>>,
        senders: DataByRank<Sender<Payload>>,
        tag: Tag,
        size: usize,
        rank: Rank,
    ) -> Self {
        Self {
            senders,
            receivers,
            rank,
            size,
            _tag: tag,
            marker_: PhantomData::default(),
        }
    }
}

impl<T> WorldCommunicator<T> for LocalCommunicator<T> {
    fn receive_vec(&mut self, _rank: Rank) -> Vec<T> {
        todo!()
    }

    fn send_vec(&mut self, _rank: Rank, _data: Vec<T>) {
        todo!()
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
