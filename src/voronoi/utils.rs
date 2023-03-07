use std::iter;

#[cfg(feature = "2d")]
pub fn sign(p1: Point, p2: Point, p3: Point) -> Float {
    use super::Point;
    use crate::prelude::Float;
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

/// Like slice.windows but including (t.last(), t.first()) as a last item.
/// Returns an empty iterator on a slice with one or zero elements.
pub fn periodic_windows<T>(v: &[T]) -> impl Iterator<Item = (&T, &T)> {
    v.iter()
        .zip(v[1..].iter().chain(iter::once(&v[0])))
        .filter(|_| v.len() > 1)
}
