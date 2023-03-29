use std::collections::HashSet;

use super::primitives::DVector;

pub struct PeriodicWindows2<'a, T> {
    values: &'a [T],
    cursor: usize,
}

impl<'a, T> Iterator for PeriodicWindows2<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.values.len() < 2 {
            return None;
        }
        let result = if self.cursor >= self.values.len() {
            None
        } else if self.cursor == self.values.len() - 1 {
            Some((&self.values[self.cursor], &self.values[0]))
        } else {
            Some((&self.values[self.cursor], &self.values[self.cursor + 1]))
        };
        self.cursor += 1;
        result
    }
}

/// A tuple version of slice.windows but including (t.last(), t.first()) as a last item.
/// Returns an empty iterator on a slice with one or zero elements.
pub fn periodic_windows_2<T>(values: &[T]) -> PeriodicWindows2<'_, T> {
    PeriodicWindows2 { values, cursor: 0 }
}

pub struct PeriodicWindows3<'a, T> {
    values: &'a [T],
    cursor: usize,
}

impl<'a, T> Iterator for PeriodicWindows3<'a, T> {
    type Item = (&'a T, &'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.values.len() < 3 || self.cursor >= self.values.len() {
            return None;
        }
        let (i, j, k) = if self.cursor == self.values.len() - 2 {
            (self.cursor, self.cursor + 1, 0)
        } else if self.cursor == self.values.len() - 1 {
            (self.cursor, 0, 1)
        } else {
            (self.cursor, self.cursor + 1, self.cursor + 2)
        };
        self.cursor += 1;
        Some((&self.values[i], &self.values[j], &self.values[k]))
    }
}

/// A tuple version of slice.windows but including (t.last(), t.first()) as a last item.
/// Returns an empty iterator on a slice with fewer than three elements.
pub fn periodic_windows_3<T>(values: &[T]) -> impl Iterator<Item = (&T, &T, &T)> {
    PeriodicWindows3 { values, cursor: 0 }
}

#[derive(Clone)]
pub struct Extent<P> {
    pub min: P,
    pub max: P,
}

pub fn get_extent_from_min_and_max_reduce<P: Clone>(
    mut vs: impl Iterator<Item = P>,
    min: fn(P, P) -> P,
    max: fn(P, P) -> P,
) -> Option<Extent<P>> {
    let v_0 = vs.next()?;
    let mut min_v = v_0.clone();
    let mut max_v = v_0;
    for v in vs {
        min_v = min(min_v, v.clone());
        max_v = max(max_v, v.clone());
    }
    Some(Extent {
        min: min_v,
        max: max_v,
    })
}

pub fn get_extent<P: DVector>(points: impl Iterator<Item = P>) -> Option<Extent<P>>
where
    P: Clone,
{
    get_extent_from_min_and_max_reduce(points, |p1, p2| P::min(p1, p2), |p1, p2| P::max(p1, p2))
}

pub struct Cyclic<'a, T> {
    items: &'a [T],
    visited: HashSet<usize>,
    visiting: usize,
    related: Box<dyn Fn(&T, &T) -> bool + 'a>,
}

/// related: a symmetric relation between two T.
/// items: a slice of items [T] where for any T_i \in [T] there are two i', i'', i' != i, i'' != i such that T_i is related to T_i' and T_i''
/// Given these parameters, iterate over pairs of items (T_i, T_j) such that
/// 1. T_i and T_j are always related
/// 2. Any item is returned exactly once as T_i and once as T_j.
/// 3. An empty iterator is returned if there are fewer than 2 items
pub fn arrange_cyclic_by<'a, T>(
    items: &'a [T],
    related: impl Fn(&T, &T) -> bool + 'a,
) -> Cyclic<'a, T> {
    Cyclic {
        items,
        visited: HashSet::default(),
        visiting: 0,
        related: Box::new(related),
    }
}

impl<'a, T> Iterator for Cyclic<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.len() < 2 || self.items.len() == self.visited.len() {
            return None;
        }
        if self.visited.len() == self.items.len() - 1 {
            let i1 = &self.items[self.visiting];
            let i2 = &self.items[0];
            assert!((self.related)(i1, i2));
            self.visited.insert(self.visiting);
            return Some((i1, i2));
        }
        let visiting = &self.items[self.visiting];
        let (related_index, related) = self
            .items
            .iter()
            .enumerate()
            .find(|(index, value)| {
                *index != self.visiting
                    && !self.visited.contains(index)
                    && (self.related)(visiting, value)
            })
            .expect("Expected another related item.");
        self.visited.insert(self.visiting);
        self.visiting = related_index;
        Some((visiting, related))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::primitives::Point2d;

    #[test]
    fn periodic_windows_2() {
        let mut w = super::periodic_windows_2(&[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(w.next().unwrap(), (&0, &1));
        assert_eq!(w.next().unwrap(), (&1, &2));
        assert_eq!(w.next().unwrap(), (&2, &3));
        assert_eq!(w.next().unwrap(), (&3, &4));
        assert_eq!(w.next().unwrap(), (&4, &5));
        assert_eq!(w.next().unwrap(), (&5, &6));
        assert_eq!(w.next().unwrap(), (&6, &7));
        assert_eq!(w.next().unwrap(), (&7, &0));
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_2(&[0, 1]);
        assert_eq!(w.next().unwrap(), (&0, &1));
        assert_eq!(w.next().unwrap(), (&1, &0));
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_2::<usize>(&[]);
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_2(&[0]);
        assert_eq!(w.next(), None);
    }

    #[test]
    fn periodic_windows_3() {
        let s = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut w = super::periodic_windows_3(&s);
        assert_eq!(w.next().unwrap(), (&0, &1, &2));
        assert_eq!(w.next().unwrap(), (&1, &2, &3));
        assert_eq!(w.next().unwrap(), (&2, &3, &4));
        assert_eq!(w.next().unwrap(), (&3, &4, &5));
        assert_eq!(w.next().unwrap(), (&4, &5, &6));
        assert_eq!(w.next().unwrap(), (&5, &6, &7));
        assert_eq!(w.next().unwrap(), (&6, &7, &0));
        assert_eq!(w.next().unwrap(), (&7, &0, &1));
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_3(&[0, 1, 2]);
        assert_eq!(w.next().unwrap(), (&0, &1, &2));
        assert_eq!(w.next().unwrap(), (&1, &2, &0));
        assert_eq!(w.next().unwrap(), (&2, &0, &1));
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_3::<usize>(&[]);
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_3(&[0]);
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows_3(&[0, 1]);
        assert_eq!(w.next(), None);
    }

    fn close(x: &usize, y: &usize) -> bool {
        let dist = ((*x as i32) - (*y as i32)).rem_euclid(7);
        dist == 1 || dist == -1 || dist == 0
    }

    #[test]
    fn arrange_cyclic_by() {
        let items = vec![3, 1, 4, 2, 5, 0, 6];
        let w: Vec<_> = super::arrange_cyclic_by(&items, close).collect();
        assert_eq!(w.len(), 7);
        for (i1, i2) in w {
            assert!(close(i1, i2));
        }
        let first_items = super::arrange_cyclic_by(&items, close)
            .map(|x| *x.0)
            .collect::<HashSet<_>>();
        let second_items = super::arrange_cyclic_by(&items, close)
            .map(|x| *x.1)
            .collect::<HashSet<_>>();
        assert_eq!(first_items.len(), 7);
        assert_eq!(second_items.len(), 7);
        assert_eq!(super::arrange_cyclic_by(&[1], close).count(), 0);
    }

    #[test]
    fn get_extent_from_min_and_max_reduce() {
        let extent = super::get_extent_from_min_and_max_reduce(
            [
                Point2d::new(0.0, 0.0),
                Point2d::new(1.0, 1.0),
                Point2d::new(2.0, 0.5),
            ]
            .into_iter(),
            Point2d::min,
            Point2d::max,
        )
        .unwrap();
        assert_float_is_close(extent.min.x, 0.0);
        assert_float_is_close(extent.min.y, 0.0);
        assert_float_is_close(extent.max.x, 2.0);
        assert_float_is_close(extent.max.y, 1.0);
        assert!(super::get_extent_from_min_and_max_reduce(
            [].into_iter(),
            Point2d::min,
            Point2d::max
        )
        .is_none());
    }
}
