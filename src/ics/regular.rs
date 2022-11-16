use super::PreSample;
use super::Sampler;
use super::SamplingData;
use crate::prelude::Float;
use crate::units::VecLength;

pub struct IntegerTuple {
    pub x: usize,
    pub y: usize,
    #[cfg(not(feature = "2d"))]
    pub z: usize,
}

#[cfg(feature = "2d")]
impl From<(usize, usize)> for IntegerTuple {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

#[cfg(not(feature = "2d"))]
impl From<(usize, usize, usize)> for IntegerTuple {
    fn from((x, y, z): (usize, usize, usize)) -> Self {
        Self { x, y, z }
    }
}

impl IntegerTuple {
    #[cfg(feature = "2d")]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    #[cfg(not(feature = "2d"))]
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    fn product(&self) -> usize {
        #[cfg(feature = "2d")]
        {
            self.x * self.y
        }
        #[cfg(not(feature = "2d"))]
        {
            self.x * self.y * self.z
        }
    }
}

pub struct RegularSampler {
    pub num_particles_per_dimension: IntegerTuple,
}

impl Sampler for RegularSampler {
    fn sample(&self, data: &SamplingData) -> PreSample {
        let volume = data.box_.volume();
        let num_particles_specified = self.num_particles_per_dimension.product();
        let positions: Vec<_> = self.get_coordinates(data).collect();
        let volume_per_particle = volume / num_particles_specified as Float;
        let masses = positions
            .iter()
            .map(|pos| data.density_profile.density(&data.box_, *pos) * volume_per_particle)
            .collect();
        PreSample { positions, masses }
    }
}

impl RegularSampler {
    pub fn new(num_particles_per_dimension: impl Into<IntegerTuple>) -> Self {
        Self {
            num_particles_per_dimension: num_particles_per_dimension.into(),
        }
    }

    #[cfg(feature = "2d")]
    fn get_coordinates<'a, 'b>(
        &'a self,
        data: &'a SamplingData,
    ) -> impl Iterator<Item = VecLength> + 'a {
        let int_coordinates_to_coordinates = |i, j| {
            let side_lengths = data.box_.side_lengths();
            VecLength::new(
                (0.5 + i as Float) * side_lengths.x() / self.num_particles_per_dimension.x as Float,
                (0.5 + j as Float) * side_lengths.y() / self.num_particles_per_dimension.y as Float,
            ) + data.box_.min
        };
        (0..self.num_particles_per_dimension.x)
            .flat_map(move |i| (0..self.num_particles_per_dimension.y).map(move |j| (i, j)))
            .map(move |(i, j)| int_coordinates_to_coordinates(i, j))
    }

    #[cfg(not(feature = "2d"))]
    fn get_coordinates<'a>(
        &'a self,
        data: &'a SamplingData,
    ) -> impl Iterator<Item = VecLength> + 'a {
        let int_coordinates_to_coordinates = |i, j, k| {
            let side_lengths = data.box_.side_lengths();
            VecLength::new(
                (0.5 + i as Float) * side_lengths.x() / self.num_particles_per_dimension.x as Float,
                (0.5 + j as Float) * side_lengths.y() / self.num_particles_per_dimension.y as Float,
                (0.5 + k as Float) * side_lengths.z() / self.num_particles_per_dimension.z as Float,
            ) + data.box_.min
        };
        (0..self.num_particles_per_dimension.x)
            .flat_map(move |i| {
                (0..self.num_particles_per_dimension.y).flat_map(move |j| {
                    (0..self.num_particles_per_dimension.z).map(move |k| (i, j, k))
                })
            })
            .map(move |(i, j, k)| int_coordinates_to_coordinates(i, j, k))
    }
}
