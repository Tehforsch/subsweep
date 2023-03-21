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

pub fn get_min_and_max<P: Clone>(
    v: &[P],
    min: fn(P, P) -> P,
    max: fn(P, P) -> P,
) -> Option<(P, P)> {
    if v.len() == 0 {
        None
    } else {
        let mut min_v = v[0].clone();
        let mut max_v = v[0].clone();
        for v in v[1..].iter() {
            min_v = min(min_v, v.clone());
            max_v = max(max_v, v.clone());
        }
        Some((min_v, max_v))
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn get_min_and_max() {
        let (min, max) = super::get_min_and_max(
            &[
                Point2d::new(0.0, 0.0),
                Point2d::new(1.0, 1.0),
                Point2d::new(2.0, 0.5),
            ],
            Point2d::min,
            Point2d::max,
        )
        .unwrap();
        assert_float_is_close(min.x, 0.0);
        assert_float_is_close(min.y, 0.0);
        assert_float_is_close(max.x, 2.0);
        assert_float_is_close(max.y, 1.0);
        assert_eq!(
            super::get_min_and_max(&[], Point2d::min, Point2d::max),
            None
        );
    }
}
