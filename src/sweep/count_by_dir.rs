use super::direction::DirectionIndex;

#[derive(Debug)]
pub struct CountByDir {
    count_by_dir: Vec<usize>,
}

impl CountByDir {
    pub fn new(num_directions: usize, value: usize) -> Self {
        let count_by_dir = (0..num_directions).map(|_| value).collect();
        Self { count_by_dir }
    }

    pub fn total(&self) -> usize {
        self.count_by_dir.iter().sum()
    }

    pub fn reduce(&mut self, dir: DirectionIndex) -> usize {
        self[dir] -= 1;
        self[dir]
    }

    /// Initialize this struct with an empty vector. Used as a sentinel value.
    pub fn empty() -> Self {
        Self {
            count_by_dir: vec![],
        }
    }
}

impl std::ops::Index<DirectionIndex> for CountByDir {
    type Output = usize;

    fn index(&self, index: DirectionIndex) -> &Self::Output {
        &self.count_by_dir[*index]
    }
}

impl std::ops::IndexMut<DirectionIndex> for CountByDir {
    fn index_mut(&mut self, index: DirectionIndex) -> &mut Self::Output {
        &mut self.count_by_dir[*index]
    }
}
