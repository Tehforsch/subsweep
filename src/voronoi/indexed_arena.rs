use std::marker::PhantomData;
use std::vec;

use generational_arena::Arena;
use generational_arena::Index;

/// This simply adds a layer of type safety around the arena, making sure
/// we cannot accidentally confuse indices into the different arenas in the
/// triangulations (i.e. use a face index for the tetra arena).
#[derive(Clone)]
pub struct IndexedArena<Id, T> {
    _marker: PhantomData<Id>,
    arena: Arena<T>,
}

impl<Id, T> Default for IndexedArena<Id, T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
            arena: Arena::default(),
        }
    }
}

impl<Id: Into<Index> + From<Index>, T> IndexedArena<Id, T> {
    pub fn get(&self, id: Id) -> Option<&T> {
        self.arena.get(id.into())
    }

    pub fn insert(&mut self, t: T) -> Id {
        self.arena.insert(t).into()
    }

    pub fn remove(&mut self, index: Id) -> Option<T> {
        self.arena.remove(index.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.arena.iter().map(|(idx, t)| (idx.into(), t))
    }

    pub fn contains(&self, id: Id) -> bool {
        self.arena.contains(id.into())
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn reserve(&mut self, cap: usize) {
        self.arena.reserve(cap)
    }

    pub fn size(&self) -> usize {
        self.arena.capacity()
    }
}

impl<Id: Into<Index> + From<Index>, T> IntoIterator for IndexedArena<Id, T> {
    type Item = T;

    type IntoIter = generational_arena::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.arena.into_iter()
    }
}

impl<Id: Into<Index> + From<Index>, T> std::ops::Index<Id> for IndexedArena<Id, T> {
    type Output = T;

    fn index(&self, index: Id) -> &Self::Output {
        &self.arena[index.into()]
    }
}

impl<Id: Into<Index> + From<Index>, T> std::ops::IndexMut<Id> for IndexedArena<Id, T> {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        &mut self.arena[index.into()]
    }
}

#[derive(Clone)]
pub struct IndexedVec<Id, T> {
    _marker: PhantomData<Id>,
    values: Vec<T>,
}

impl<Id, T> Default for IndexedVec<Id, T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
            values: vec![],
        }
    }
}

impl<Id: Into<usize> + From<usize>, T> IndexedVec<Id, T> {
    pub fn insert(&mut self, t: T) -> Id {
        self.values.push(t);
        (self.values.len() - 1).into()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.values
            .iter()
            .enumerate()
            .map(|(idx, t)| (idx.into(), t))
    }

    pub fn reserve(&mut self, cap: usize) {
        self.values.reserve(cap)
    }
}

impl<Id: Into<usize> + From<usize>, T> IntoIterator for IndexedVec<Id, T> {
    type Item = T;

    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<Id: Into<usize> + From<usize>, T> std::ops::Index<Id> for IndexedVec<Id, T> {
    type Output = T;

    fn index(&self, index: Id) -> &Self::Output {
        &self.values[index.into()]
    }
}

impl<Id: Into<usize> + From<usize>, T> std::ops::IndexMut<Id> for IndexedVec<Id, T> {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        &mut self.values[index.into()]
    }
}
