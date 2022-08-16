use std::collections::hash_map;
use std::collections::HashMap;

use mpi::Rank;

pub struct DataByRank<T>(HashMap<Rank, T>);

impl<T> DataByRank<T>
where
    T: Default,
{
    pub fn new(num_ranks: usize, this_rank: Rank) -> Self {
        Self(
            (0..num_ranks)
                .filter(|rank| *rank != this_rank as usize)
                .map(|rank| (rank as Rank, T::default()))
                .collect(),
        )
    }
}

impl<T> DataByRank<Vec<T>> {
    pub fn push(&mut self, rank: Rank, data: T) {
        self.0.get_mut(&rank).unwrap().push(data);
    }
}

impl<T> DataByRank<T> {
    pub fn get(&self, rank: &Rank) -> Option<&T> {
        self.0.get(rank)
    }

    pub fn insert(&mut self, rank: Rank, data: T) {
        self.0.insert(rank, data);
    }
}

impl<T> IntoIterator for DataByRank<T> {
    type Item = (Rank, T);

    type IntoIter = hash_map::IntoIter<Rank, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait BufferedCommunicator<T> {
    fn send(&mut self, rank: i32, data: T);
    fn receive_vec(self) -> DataByRank<Vec<T>>;
}
