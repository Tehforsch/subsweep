use std::collections::hash_map;
use std::collections::HashMap;

use mpi::Rank;

pub struct DataByRank<T>(HashMap<Rank, T>);

impl<T> Clone for DataByRank<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> DataByRank<T> {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }
}

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

    pub fn drain_all(&mut self) -> impl Iterator<Item = (Rank, Vec<T>)> + '_ {
        self.0.iter_mut().map(|(k, v)| (*k, v.drain(..).collect()))
    }
}

impl<T> DataByRank<T> {
    #[cfg(feature = "local")]
    pub fn get(&self, rank: &Rank) -> Option<&T> {
        self.0.get(rank)
    }

    #[cfg(feature = "local")]
    pub fn remove(&mut self, rank: &Rank) -> Option<T> {
        self.0.remove(rank)
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
