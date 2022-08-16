use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::Communicator;
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

impl<T> Communicator<T> for LocalCommunicator<T> {
    fn receive_vec(&mut self, rank: Rank) -> Vec<T> {
        self.receivers.get(&rank).unwrap().recv().unwrap()
    }

    fn send_vec(&mut self, rank: Rank, data: Vec<T>) {
        self.senders.get(&rank).unwrap().send(data).unwrap();
    }

    fn rank(&self) -> Rank {
        self.rank
    }

    fn size(&self) -> usize {
        self.size
    }
}
