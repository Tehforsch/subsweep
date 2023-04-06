use core::fmt::Debug;
use std::ops::Index;
use std::ops::IndexMut;

use mpi::Rank;

use super::SizedCommunicator;
use crate::hash_map::HashMap;

pub struct DataByRank<T>(HashMap<Rank, T>);

impl<T> Default for DataByRank<T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

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
    pub fn empty() -> Self {
        Self(HashMap::default())
    }
}

impl<T> DataByRank<Vec<T>> {
    pub fn size(&self) -> usize {
        self.0.values().map(|x| x.len()).sum()
    }
}

impl<T> DataByRank<T>
where
    T: Default,
{
    pub fn from_communicator(communicator: &impl SizedCommunicator) -> Self {
        Self::from_size_and_rank(communicator.size(), communicator.rank())
    }

    pub fn from_size_and_rank(size: usize, this_rank: Rank) -> Self {
        Self(
            (0..size)
                .filter(|rank| *rank != this_rank as usize)
                .map(|rank| (rank as Rank, T::default()))
                .collect(),
        )
    }

    pub fn drain_all(&mut self) -> impl Iterator<Item = (Rank, T)> + '_ {
        self.0.iter_mut().map(|(k, v)| {
            let mut swapped = T::default();
            std::mem::swap(&mut swapped, v);
            (*k, swapped)
        })
    }
}

impl<T: Clone> DataByRank<T> {
    pub fn same_from_size_and_rank(t: T, size: usize, this_rank: Rank) -> Self {
        Self(
            (0..size)
                .filter(|rank| *rank != this_rank as usize)
                .map(|rank| (rank as Rank, t.clone()))
                .collect(),
        )
    }

    pub fn same_for_all_ranks_in_communicator(t: T, communicator: &impl SizedCommunicator) -> Self {
        Self::same_from_size_and_rank(t, communicator.size(), communicator.rank())
    }
}

impl<T> DataByRank<Vec<T>> {
    pub fn push(&mut self, rank: Rank, item: T) {
        self[rank].push(item);
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

    pub fn remove(&mut self, rank: &Rank) -> Option<T> {
        self.0.remove(rank)
    }

    pub fn insert(&mut self, rank: Rank, data: T) {
        self.0.insert(rank, data);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Rank, &T)> + '_ {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Rank, &mut T)> + '_ {
        self.0.iter_mut()
    }
}

impl<T> IntoIterator for DataByRank<T> {
    type Item = (Rank, T);

    type IntoIter = <HashMap<Rank, T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> FromIterator<(Rank, T)> for DataByRank<T> {
    fn from_iter<I: IntoIterator<Item = (Rank, T)>>(iter: I) -> Self {
        Self(HashMap::from_iter(iter))
    }
}
