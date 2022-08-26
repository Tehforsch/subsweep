use core::fmt::Debug;
use std::collections::hash_map;
use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;

use mpi::Rank;

use super::SizedCommunicator;

pub struct DataByRank<T>(HashMap<Rank, T>);

impl<T> Debug for DataByRank<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Clone for DataByRank<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> DataByRank<T> {
    #[cfg(any(feature = "local", test))]
    pub fn empty() -> Self {
        Self(HashMap::new())
    }
}

impl<T> DataByRank<T>
where
    T: Default,
{
    pub fn from_communicator(communicator: &impl SizedCommunicator) -> Self {
        Self(
            (0..communicator.size())
                .filter(|rank| *rank != communicator.rank() as usize)
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

    pub fn drain_all_sorted(&mut self) -> impl Iterator<Item = (Rank, Vec<T>)> + '_ {
        let mut keys: Vec<_> = self.0.keys().map(|k| *k).collect();
        keys.sort();
        keys.into_iter().map(|k| (k, self.0.remove(&k).unwrap()))
    }
}

impl<T> Index<Rank> for DataByRank<T> {
    type Output = T;

    fn index(&self, index: Rank) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<T> IndexMut<Rank> for DataByRank<T> {
    fn index_mut(&mut self, index: Rank) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}

impl<T> DataByRank<T> {
    pub fn get(&self, rank: &Rank) -> Option<&T> {
        self.0.get(rank)
    }

    pub fn get_mut(&mut self, rank: &Rank) -> Option<&mut T> {
        self.0.get_mut(rank)
    }

    #[cfg(feature = "local")]
    #[cfg(test)]
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
