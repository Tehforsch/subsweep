use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use super::Dimension;
use super::Point3d;
use super::ThreeD;
use super::TwoD;
use crate::prelude::ParticleId;
use crate::voronoi::Point2d;

pub trait TestDimension: Dimension {
    fn num() -> usize;
    fn get_example_point_set(shift: usize) -> Vec<Self::Point> {
        Self::get_example_point_set_num(100, shift)
    }

    fn get_example_point_set_num(num: usize, shift: usize) -> Vec<Self::Point>;
    fn number_of_tetras(num_inserted_points: usize) -> Option<usize>;
    fn number_of_faces(num_inserted_points: usize) -> Option<usize>;
    fn number_of_points(num_inserted_points: usize) -> Option<usize>;
    fn get_lookup_points() -> Vec<Self::Point>;

    fn get_combined_point_set() -> Vec<(ParticleId, Self::Point)> {
        let (p1, p2) = Self::get_example_point_sets_with_ids();
        p1.into_iter().chain(p2.into_iter()).collect()
    }

    fn get_example_point_sets_with_ids() -> (
        Vec<(ParticleId, Self::Point)>,
        Vec<(ParticleId, Self::Point)>,
    ) {
        let (p1, p2) = (
            Self::get_example_point_set(0),
            Self::get_example_point_set(1),
        );
        let len_p1 = p1.len();
        (
            p1.into_iter()
                .enumerate()
                .map(|(i, p)| (ParticleId(i as u64), p))
                .collect(),
            p2.into_iter()
                .enumerate()
                .map(|(i, p)| (ParticleId(len_p1 as u64 + i as u64), p))
                .collect(),
        )
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

    fn get_example_point_set_num(num: usize, shift: usize) -> Vec<Self::Point> {
        let mut rng = StdRng::seed_from_u64(1338 + shift as u64);
        (0..num)
            .map(|_| {
                let offset = 0.3 * shift as f64;
                let x = rng.gen_range(0.1 + offset..0.4 + offset);
                let y = rng.gen_range(0.1..0.4);
                Point2d::new(x, y)
            })
            .collect()
    }

    fn get_lookup_points() -> Vec<Point2d> {
        (0..30)
            .flat_map(move |i| (0..30).map(move |j| (i, j)))
            .map(|(i, j)| Point2d::new(0.01 * i as f64, 0.01 * j as f64))
            .collect()
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

    fn get_example_point_set_num(num: usize, shift: usize) -> Vec<Point3d> {
        let mut rng = StdRng::seed_from_u64(1338 + shift as u64);
        (0..num)
            .map(|_| {
                let offset = 0.3 * shift as f64;
                let x = rng.gen_range(0.1 + offset..0.4 + offset);
                let y = rng.gen_range(0.1..0.4);
                let z = rng.gen_range(0.1..0.4);
                Point3d::new(x, y, z)
            })
            .collect()
    }

    fn get_lookup_points() -> Vec<Point3d> {
        (0..10)
            .flat_map(move |i| (0..10).map(move |j| (i, j)))
            .flat_map(move |(i, j)| (0..10).map(move |k| (i, j, k)))
            .map(|(i, j, k)| Point3d::new(0.1 * i as f64, 0.1 * j as f64, 0.1 * k as f64))
            .collect()
    }
}
