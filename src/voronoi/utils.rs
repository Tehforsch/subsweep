use std::iter;

#[cfg(feature = "2d")]
pub fn sign(p1: super::Point, p2: super::Point, p3: super::Point) -> crate::prelude::Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

/// A tuple version of slice.windows but including (t.last(), t.first()) as a last item.
/// Returns an empty iterator on a slice with one or zero elements.
pub fn periodic_windows<T>(v: &[T]) -> impl Iterator<Item = (&T, &T)> {
    v.iter()
        .zip(v[1..].iter().chain(iter::once(&v[0])))
        .filter(|_| v.len() > 1)
}

/// A tuple version of slice.windows but including (t.last(), t.first()) as a last item.
/// Returns an empty iterator on a slice with fewer than three elements.
#[cfg(feature = "3d")]
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
        let s = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut w = super::periodic_windows(&s);
        assert_eq!(w.next().unwrap(), (&0, &1));
        assert_eq!(w.next().unwrap(), (&1, &2));
        assert_eq!(w.next().unwrap(), (&2, &3));
        assert_eq!(w.next().unwrap(), (&3, &4));
        assert_eq!(w.next().unwrap(), (&4, &5));
        assert_eq!(w.next().unwrap(), (&5, &6));
        assert_eq!(w.next().unwrap(), (&6, &7));
        assert_eq!(w.next().unwrap(), (&7, &0));
        assert_eq!(w.next(), None);
    }

    #[test]
    #[cfg(feature = "3d")]
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
    }
}
