use std::iter;

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
pub fn periodic_windows<T>(values: &[T]) -> PeriodicWindows2<'_, T> {
    PeriodicWindows2 { values, cursor: 0 }
}

/// A tuple version of slice.windows but including (t.last(), t.first()) as a last item.
/// Returns an empty iterator on a slice with fewer than three elements.
pub fn periodic_windows_3<T>(v: &[T]) -> impl Iterator<Item = (&T, &T, &T)> {
    v.iter()
        .zip(v[1..].iter().chain(iter::once(&v[0])))
        .zip(
            v[2..]
                .iter()
                .chain(iter::once(&v[0]))
                .chain(iter::once(&v[1])),
        )
        .map(|((v1, v2), v3)| (v1, v2, v3))
        .filter(|_| v.len() > 2)
}

#[cfg(test)]
mod tests {
    #[test]
    fn periodic_windows_2() {
        let mut w = super::periodic_windows(&[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(w.next().unwrap(), (&0, &1));
        assert_eq!(w.next().unwrap(), (&1, &2));
        assert_eq!(w.next().unwrap(), (&2, &3));
        assert_eq!(w.next().unwrap(), (&3, &4));
        assert_eq!(w.next().unwrap(), (&4, &5));
        assert_eq!(w.next().unwrap(), (&5, &6));
        assert_eq!(w.next().unwrap(), (&6, &7));
        assert_eq!(w.next().unwrap(), (&7, &0));
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows(&[0, 1]);
        assert_eq!(w.next().unwrap(), (&0, &1));
        assert_eq!(w.next().unwrap(), (&1, &0));
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows::<usize>(&[]);
        assert_eq!(w.next(), None);
        let mut w = super::periodic_windows(&[0]);
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
        todo!("fix actual implementation and test")
    }
}
