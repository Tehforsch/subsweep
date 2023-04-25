use std::ops::Add;
use std::ops::Div;
use std::ops::Sub;

use mpi::datatype::UserDatatype;
use mpi::internal::memoffset::offset_of;
use mpi::traits::Equivalence;
use mpi::Address;
use serde::Serialize;

use crate::voronoi::DVector;
use crate::voronoi::MinMax;

#[derive(Clone, Serialize, Default)]
pub struct Extent<P> {
    pub min: P,
    pub max: P,
    #[serde(skip_serializing)]
    pub center: P,
}

impl<P: Add<P, Output = P> + Sub<P, Output = P> + Div<f64, Output = P> + Clone + Copy> Extent<P> {
    pub fn from_min_max(min: P, max: P) -> Extent<P> {
        let center: P = (min + max) / 2.0;
        Extent { min, max, center }
    }

    /// Return a new extent which is three times larger
    /// so that it includes all (first-order) periodic images of the particles
    /// in the original extent.
    pub fn including_periodic_images(&self) -> Self {
        let dist = self.max - self.min;
        Self::from_min_max(self.min - dist, self.max + dist)
    }
}

impl<P> Extent<P>
where
    P: DVector + Copy,
{
    pub fn max_side_length(&self) -> P::Float {
        let side_length = self.max - self.min;
        side_length.max_element()
    }
}

pub fn get_extent_from_min_and_max_reduce<
    P: Clone + Div<f64, Output = P> + Add<P, Output = P> + Sub<P, Output = P> + Clone + Copy,
>(
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
    Some(Extent::from_min_max(min_v, max_v))
}

pub fn get_extent<P>(points: impl Iterator<Item = P>) -> Option<Extent<P>>
where
    P: MinMax
        + Clone
        + Div<f64, Output = P>
        + Add<P, Output = P>
        + Sub<P, Output = P>
        + Clone
        + Copy,
{
    get_extent_from_min_and_max_reduce(points, |p1, p2| P::min(p1, p2), |p1, p2| P::max(p1, p2))
}

unsafe impl<P> Equivalence for Extent<P>
where
    P: Equivalence,
{
    type Out = UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        UserDatatype::structured(
            &[1, 1, 1],
            &[
                offset_of!(Self, min) as Address,
                offset_of!(Self, max) as Address,
                offset_of!(Self, center) as Address,
            ],
            &[
                UserDatatype::contiguous(1, &P::equivalent_datatype()),
                UserDatatype::contiguous(1, &P::equivalent_datatype()),
                UserDatatype::contiguous(1, &P::equivalent_datatype()),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::Point2d;

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
