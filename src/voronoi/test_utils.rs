use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use super::delaunay::Point;
use super::delaunay::TetraData;
use super::math::utils::determinant3x3_sign;
use super::math::utils::determinant4x4_sign;
use super::DDimension;
use super::Point3d;
use crate::dimension::ThreeD;
use crate::dimension::TwoD;
use crate::extent::Extent;
use crate::voronoi::Point2d;

pub trait TestDimension: DDimension {
    fn num() -> usize;
    fn number_of_tetras(num_inserted_points: usize) -> Option<usize>;
    fn number_of_faces(num_inserted_points: usize) -> Option<usize>;
    fn number_of_points(num_inserted_points: usize) -> Option<usize>;

    fn test_extent(y_offset: f64) -> Extent<Self::Point>;
    fn get_points_in_extent(
        extent: &Extent<Self::Point>,
        seed: u64,
    ) -> Box<dyn Iterator<Item = Self::Point> + '_>;
    fn tetra_is_positively_oriented(t: &TetraData<Self>) -> bool;
    fn extent_contains(extent: &Extent<Self::Point>, point: &Self::Point) -> bool;

    fn get_lookup_points() -> Vec<Self::Point> {
        Self::get_points_in_extent(&Self::test_extent(0.0), 1991)
            .take(100)
            .collect()
    }

    fn get_example_point_set_num(num: usize, shift: usize) -> Vec<Self::Point> {
        let seed = 1338 + shift as u64;
        let extent = Self::test_extent(shift as f64 * 0.3);
        Self::get_points_in_extent(&extent, seed)
            .take(num)
            .collect()
    }

    fn get_surrounding_points(num: usize) -> Vec<Self::Point> {
        let seed = 1438;
        let extent = Self::test_extent(0.0);
        let periodic_extent = Self::test_extent(0.0).including_periodic_images();
        Self::get_points_in_extent(&periodic_extent, seed)
            .filter(|p| !Self::extent_contains(&extent, &p))
            .take(num)
            .collect()
    }
}

impl TestDimension for TwoD {
    fn number_of_tetras(num_inserted_points: usize) -> Option<usize> {
        Some(1 + 2 * num_inserted_points)
    }

    fn number_of_faces(num_inserted_points: usize) -> Option<usize> {
        Some(3 + 3 * num_inserted_points)
    }

    fn number_of_points(num_inserted_points: usize) -> Option<usize> {
        Some(3 + num_inserted_points)
    }

    fn num() -> usize {
        2
    }

    fn extent_contains(extent: &Extent<Self::Point>, point: &Self::Point) -> bool {
        extent.contains(point)
    }

    fn test_extent(x_offset: f64) -> Extent<Point2d> {
        let min = Point2d::new(0.1 + x_offset, 0.1);
        let max = Point2d::new(0.4 + x_offset, 0.4);
        Extent::from_min_max(min, max)
    }

    fn get_points_in_extent(
        extent: &Extent<Point<TwoD>>,
        seed: u64,
    ) -> Box<dyn Iterator<Item = Self::Point> + '_> {
        let mut rng = StdRng::seed_from_u64(seed);
        Box::new(std::iter::from_fn(move || {
            let x = rng.gen_range(extent.min.x..extent.max.x);
            let y = rng.gen_range(extent.min.y..extent.max.y);
            Some(Point2d::new(x, y))
        }))
    }

    #[rustfmt::skip]
    fn tetra_is_positively_oriented(t: &TetraData<TwoD>) -> bool {
        let sign = determinant3x3_sign(
            [
                [1.0, t.p1.x, t.p1.y],
                [1.0, t.p2.x, t.p2.y],
                [1.0, t.p3.x, t.p3.y],
            ]
        );
        sign.panic_if_zero(|| "Zero volume tetra encountered").is_positive()
    }
}

impl TestDimension for ThreeD {
    fn num() -> usize {
        3
    }

    // In 3d we don't know how many tetras/faces there should be at any given level
    // because of 2-to-3 flips and 3-to-2 flips
    fn number_of_tetras(_: usize) -> Option<usize> {
        None
    }

    fn number_of_faces(_: usize) -> Option<usize> {
        None
    }

    fn number_of_points(num_inserted_points: usize) -> Option<usize> {
        Some(4 + num_inserted_points)
    }

    fn extent_contains(extent: &Extent<Self::Point>, point: &Self::Point) -> bool {
        extent.contains(point)
    }

    fn test_extent(x_offset: f64) -> Extent<Point3d> {
        let min = Point3d::new(0.1 + x_offset, 0.1, 0.1);
        let max = Point3d::new(0.4 + x_offset, 0.4, 0.4);
        Extent::from_min_max(min, max)
    }

    fn get_points_in_extent(
        extent: &Extent<Point3d>,
        seed: u64,
    ) -> Box<dyn Iterator<Item = Self::Point> + '_> {
        let mut rng = StdRng::seed_from_u64(seed);
        Box::new(std::iter::from_fn(move || {
            let x = rng.gen_range(extent.min.x..extent.max.x);
            let y = rng.gen_range(extent.min.y..extent.max.y);
            let z = rng.gen_range(extent.min.z..extent.max.z);
            Some(Point3d::new(x, y, z))
        }))
    }

    #[rustfmt::skip]
    fn tetra_is_positively_oriented(t: &TetraData<ThreeD>) -> bool {
        determinant4x4_sign(
            [
                [1.0, t.p1.x, t.p1.y, t.p1.z],
                [1.0, t.p2.x, t.p2.y, t.p2.z],
                [1.0, t.p3.x, t.p3.y, t.p3.z],
                [1.0, t.p4.x, t.p4.y, t.p4.z],
            ]
        ).panic_if_zero(|| "Zero volume tetra encountered").is_positive()
    }
}
