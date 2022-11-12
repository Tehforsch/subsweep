use bevy::prelude::Commands;

use super::Sample;
use super::VelocityProfile;
use super::ZeroVelocity;
use crate::prelude::Float;
use crate::prelude::SimulationBox;
use crate::units::Density;
use crate::units::VecLength;

pub struct IntegerTuple {
    x: usize,
    y: usize,
    #[cfg(not(feature = "2d"))]
    z: usize,
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
    density: Density,
    box_size: SimulationBox,
    num_particles_per_dimension: IntegerTuple,
    velocity_profile: Box<dyn VelocityProfile>,
}

impl RegularSampler {
    pub fn new(
        density: Density,
        box_size: SimulationBox,
        num_particles_per_dimension: IntegerTuple,
    ) -> Self {
        Self {
            density,
            box_size,
            num_particles_per_dimension,
            velocity_profile: Box::new(ZeroVelocity),
        }
    }

    pub fn velocity_profile(self, velocity_profile: impl VelocityProfile + 'static) -> Self {
        Self {
            velocity_profile: Box::new(velocity_profile),
            ..self
        }
    }

    pub fn sample(&mut self) -> Sample {
        let volume = self.box_size.volume();
        let total_mass = self.density * volume;
        let num_particles_specified = self.num_particles_per_dimension.product();
        let mass_per_particle = total_mass / num_particles_specified as Float;
        let positions = self.get_coordinates().collect();
        Sample {
            positions,
            mass_per_particle,
        }
    }

    pub fn spawn(mut self, commands: &mut Commands) {
        self.sample().spawn(commands, &*self.velocity_profile)
    }

    #[cfg(feature = "2d")]
    fn get_coordinates(&self) -> impl Iterator<Item = VecLength> + '_ {
        let int_coordinates_to_coordinates = |i, j| {
            let side_lengths = self.box_size.side_lengths();
            VecLength::new(
                (0.5 + i as Float) * side_lengths.x() / self.num_particles_per_dimension.x as Float,
                (0.5 + j as Float) * side_lengths.y() / self.num_particles_per_dimension.y as Float,
            ) + self.box_size.min
        };
        (0..self.num_particles_per_dimension.x)
            .flat_map(move |i| (0..self.num_particles_per_dimension.y).map(move |j| (i, j)))
            .map(move |(i, j)| int_coordinates_to_coordinates(i, j))
    }

    #[cfg(not(feature = "2d"))]
    fn get_coordinates(&self) -> impl Iterator<Item = VecLength> + '_ {
        let int_coordinates_to_coordinates = |i, j, k| {
            let side_lengths = self.box_size.side_lengths();
            VecLength::new(
                (0.5 + i as Float) * side_lengths.x() / self.num_particles_per_dimension.x as Float,
                (0.5 + j as Float) * side_lengths.y() / self.num_particles_per_dimension.y as Float,
                (0.5 + k as Float) * side_lengths.z() / self.num_particles_per_dimension.z as Float,
            ) + self.box_size.min
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
