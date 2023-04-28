use std::marker::PhantomData;

use mpi::request::scope;
use mpi::traits::Equivalence;

use super::communicator::Communicator;
use super::DataByRank;
use super::MpiWorld;
use super::Rank;
use super::SizedCommunicator;

pub struct ExchangeCommunicator<T> {
    pub communicator: MpiWorld<T>,
    pending_data: DataByRank<bool>,
    _marker: PhantomData<T>,
}

impl<T: 'static> ExchangeCommunicator<T> {
    pub fn new() -> Self {
        let communicator = MpiWorld::new();
        let pending_data = DataByRank::from_communicator(&communicator);
        Self {
            communicator,
            pending_data,
            _marker: PhantomData::default(),
        }
    }
}

impl<T> SizedCommunicator for ExchangeCommunicator<T> {
    fn size(&self) -> usize {
        self.communicator.size()
    }

    fn rank(&self) -> Rank {
        self.communicator.rank()
    }
}

impl<T> From<Communicator<T>> for ExchangeCommunicator<T> {
    fn from(communicator: Communicator<T>) -> Self {
        let pending_data = DataByRank::from_communicator(&communicator);
        Self {
            communicator,
            pending_data,
            _marker: PhantomData::default(),
        }
    }
}

impl<T> ExchangeCommunicator<T>
where
    T: Equivalence,
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
                debug_assert!(!self.pending_data[rank]);
                self.pending_data[rank] = true;
                let guard = self
                    .communicator
                    .immediate_send_vec_wait_guard(scope, rank, items);
                guards.extend(guard.into_iter());
            }
            self.receive_vec()
        })
    }

    pub fn receive_vec(&mut self) -> DataByRank<Vec<T>> {
        self.empty_send_to_others();
        let mut received_data = DataByRank::from_communicator(&self.communicator);
        for rank in self.communicator.other_ranks() {
            debug_assert!(self.pending_data[rank]);
        }
        for rank in self.communicator.other_ranks() {
            let received = self.communicator.receive_vec(rank);
            received_data.insert(rank, received);
            self.pending_data[rank] = false;
        }
        received_data
    }
}
