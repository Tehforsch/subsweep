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
            _marker: PhantomData,
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
            _marker: PhantomData,
        }
    }
}

impl<T> ExchangeCommunicator<T>
where
    T: Equivalence,
{
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

    pub fn exchange_all<U: AsRef<[T]>>(&mut self, data: DataByRank<U>) -> DataByRank<Vec<T>> {
        scope(|scope| {
            let mut guards = vec![];
            for (rank, items) in data.iter() {
                debug_assert!(!self.pending_data[rank]);
                self.pending_data[rank] = true;
                let guard =
                    self.communicator
                        .immediate_send_vec_wait_guard(scope, rank, items.as_ref());
                guards.extend(guard.into_iter());
            }
            self.receive_vec()
        })
    }

    pub fn exchange_same_for_all(&mut self, data: &[T]) -> DataByRank<Vec<T>> {
        self.exchange_all(
            self.other_ranks()
                .iter()
                .map(|rank| (*rank, data))
                .collect(),
        )
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

pub fn divide_into_chunks_with_same_num_globally<T>(
    items: &[T],
    chunk_size: usize,
) -> impl Iterator<Item = &[T]> + '_ {
    let num_chunks = global_num_chunks(items.len(), chunk_size);
    let mut chunk_iter = items.chunks(chunk_size);
    (0..num_chunks).map(move |_| chunk_iter.next().unwrap_or(&[]))
}

fn global_num_chunks(num_elements: usize, chunk_size: usize) -> usize {
    let mut comm: Communicator<usize> = Communicator::new();
    let num_chunks = div_ceil(num_elements, chunk_size);
    comm.all_gather_max(&num_chunks).unwrap()
}

fn div_ceil(x: usize, y: usize) -> usize {
    (x / y) + if x.rem_euclid(y) > 0 { 1 } else { 0 }
}
