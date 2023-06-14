use core::fmt::Debug;
use std::ops::Index;
use std::ops::IndexMut;

use mpi::Rank;

use super::SizedCommunicator;

pub struct DataByRank<T>(Vec<Option<T>>);

impl<T> Default for DataByRank<T> {
    fn default() -> Self {
        Self(vec![])
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
        Self(vec![])
    }
}

impl<T> DataByRank<Vec<T>> {
    pub fn size(&self) -> usize {
        self.0
            .iter()
            .filter_map(|t| t.as_ref().map(|x| x.len()))
            .sum()
    }
}

impl<T> DataByRank<T> {
    pub fn from_closure_size_and_rank(f: impl Fn() -> T, size: usize, this_rank: Rank) -> Self {
        let items = (0..size)
            .map(|rank| {
                if rank as Rank == this_rank {
                    None
                } else {
                    Some(f())
                }
            })
            .collect();
        Self(items)
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
        Self::from_closure_size_and_rank(|| T::default(), size, this_rank)
    }

    pub fn drain_all(&mut self) -> impl Iterator<Item = (Rank, T)> + '_ {
        self.0
            .drain(..)
            .enumerate()
            .filter_map(|(rank, t)| t.map(|t| (rank as Rank, t)))
    }
}

impl<T: Clone> DataByRank<T> {
    pub fn same_for_other_ranks_in_communicator(
        t: T,
        communicator: &impl SizedCommunicator,
    ) -> Self {
        Self::from_closure_size_and_rank(|| t.clone(), communicator.size(), communicator.rank())
    }

    pub fn same_for_all_ranks_in_communicator(t: T, communicator: &impl SizedCommunicator) -> Self {
        Self::from_closure_size_and_rank(|| t.clone(), communicator.size(), -1)
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
        self.0[*rank as usize].as_ref()
    }

    pub fn get_mut(&mut self, rank: &Rank) -> Option<&mut T> {
        self.0[*rank as usize].as_mut()
    }

    pub fn remove(&mut self, rank: &Rank) -> Option<T> {
        self.0[*rank as usize].take()
    }

    pub fn insert(&mut self, rank: Rank, data: T) {
        let rank = rank as usize;
        if rank >= self.0.len() {
            self.extend(rank - self.0.len() + 1);
        }
        self.0[rank] = Some(data);
    }

    fn extend(&mut self, num: usize) {
        for _ in 0..num {
            self.0.push(None);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Rank, &T)> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(i, t)| t.as_ref().map(|t| (i as Rank, t)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Rank, &mut T)> + '_ {
        self.0
            .iter_mut()
            .enumerate()
            .filter_map(|(i, t)| t.as_mut().map(|t| (i as Rank, t)))
    }
}

impl<T> IntoIterator for DataByRank<T> {
    type Item = (Rank, T);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }

    type IntoIter = IntoIter<T>;
}

pub struct IntoIter<T> {
    data: DataByRank<T>,
    cursor: i32,
}

impl<T> IntoIter<T> {
    fn new(data: DataByRank<T>) -> Self {
        Self { data, cursor: 0 }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = (Rank, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.data.0.len() as i32 {
            let item = self.data.remove(&self.cursor);
            self.cursor += 1;
            if let Some(item) = item {
                return Some((self.cursor - 1, item));
            }
        }
        None
    }
}

impl<T> FromIterator<(Rank, T)> for DataByRank<T> {
    fn from_iter<I: IntoIterator<Item = (Rank, T)>>(iter: I) -> Self {
        let mut items = Self::empty();
        for (k, v) in iter {
            items.insert(k, v);
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::DataByRank;

    #[test]
    fn into_iter() {
        let mut x: DataByRank<f64> = DataByRank::empty();
        x.insert(0, 10.0);
        x.insert(1, 20.0);
        x.insert(3, 30.0);
        let mut iter = x.into_iter();
        assert_eq!(iter.next(), Some((0, 10.0)));
        assert_eq!(iter.next(), Some((1, 20.0)));
        assert_eq!(iter.next(), Some((3, 30.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn from_iter() {
        let a = [(0, 10.0), (1, 20.0), (3, 30.0)];
        let x: DataByRank<f64> = a.into_iter().collect();
        let mut iter = x.into_iter();
        assert_eq!(iter.next(), Some((0, 10.0)));
        assert_eq!(iter.next(), Some((1, 20.0)));
        assert_eq!(iter.next(), Some((3, 30.0)));
        assert_eq!(iter.next(), None);
    }
}
