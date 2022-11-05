mod density_profile;
mod resolution;
mod velocity_profile;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::Commands;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub use self::density_profile::ConstantDensity;
pub use self::density_profile::DensityProfile;
pub use self::resolution::Resolution;
pub use self::velocity_profile::ConstantVelocity;
pub use self::velocity_profile::VelocityProfile;
pub use self::velocity_profile::ZeroVelocity;
use crate::components;
use crate::components::Position;
use crate::components::Velocity;
use crate::parameters::BoxSize;
use crate::prelude::Float;
use crate::prelude::LocalParticle;
use crate::rand::gen_range;
use crate::units::Density;
use crate::units::Mass;
use crate::units::VecLength;

pub const DEFAULT_SEED: u64 = 123;

pub struct Sample {
    positions: Vec<VecLength>,
    mass_per_particle: Mass,
}

pub struct Sampler {
    density_profile: Box<dyn DensityProfile>,
    velocity_profile: Box<dyn VelocityProfile>,
    box_size: BoxSize,
    num_samples: usize,
    resolution: Resolution,
    rng: StdRng,
}

impl Sampler {
    pub fn new(
        density_profile: impl DensityProfile + 'static,
        box_size: &BoxSize,
        resolution: Resolution,
    ) -> Self {
        Self {
            density_profile: Box::new(density_profile),
            velocity_profile: Box::new(ZeroVelocity),
            box_size: box_size.clone(),
            resolution,
            num_samples: 100000,
            rng: StdRng::seed_from_u64(DEFAULT_SEED),
        }
    }

    pub fn velocity_profile(self, velocity_profile: impl VelocityProfile + 'static) -> Self {
        Self {
            velocity_profile: Box::new(velocity_profile),
            ..self
        }
    }

    pub fn spawn(self, commands: &mut Commands) {
        self.spawn_with(commands, |_, _, _| {})
    }

    pub fn spawn_with(
        mut self,
        commands: &mut Commands,
        spawn_additional_components: impl Fn(&mut EntityCommands, VecLength, Mass),
    ) {
        let sample = self.sample();
        for pos in sample.positions.into_iter() {
            let velocity = self.velocity_profile.velocity(pos);
            let mut entity_commands = commands.spawn_bundle((
                LocalParticle,
                Position(pos),
                components::Mass(sample.mass_per_particle),
                Velocity(velocity),
            ));
            spawn_additional_components(&mut entity_commands, pos, sample.mass_per_particle);
        }
    }

    fn sample(&mut self) -> Sample {
        let total_mass_profile = self.integrate();
        let volume = self.box_size.volume();
        let num_particles_specified =
            (self.resolution.as_number_density(volume) * volume).value() as usize;
        let mass_per_particle = total_mass_profile / num_particles_specified as Float;
        let mut positions = vec![];
        // A simple implementation of rejection sampling
        while positions.len() < num_particles_specified {
            let pos = gen_range(&mut self.rng, self.box_size.min, self.box_size.max);
            let random_density = self
                .rng
                .gen_range(Density::zero()..self.density_profile.max_value());
            if random_density < self.density_profile.density(pos) {
                positions.push(pos);
            }
        }
        Sample {
            positions,
            mass_per_particle,
        }
    }

    fn integrate(&mut self) -> Mass {
        let volume_per_sample = self.box_size.volume() / (self.num_samples as Float);
        let mut mass = Mass::zero();
        for _ in 0..self.num_samples {
            let pos = gen_range(&mut self.rng, self.box_size.min, self.box_size.max);
            mass += self.density_profile.density(pos) * volume_per_sample;
        }
        mass
    }
}
