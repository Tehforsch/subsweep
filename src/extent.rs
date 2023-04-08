use crate::voronoi::DVector;

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
