use bevy::ecs::system::EntityCommands;
use bevy::prelude::Commands;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::components;
use crate::components::Position;
use crate::domain::Extent;
use crate::prelude::Float;
use crate::prelude::LocalParticle;
use crate::rand::gen_range;
use crate::units::Density;
use crate::units::Mass;
use crate::units::NumberDensity;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::units::Volume;

pub const DEFAULT_SEED: u64 = 123;

pub enum ResolutionSpecification {
    NumberDensity(NumberDensity),
    NumParticles(usize),
}

impl ResolutionSpecification {
    fn as_number_density(&self, volume: Volume) -> NumberDensity {
        match self {
            Self::NumberDensity(density) => *density,
            Self::NumParticles(num) => *num as Float / volume,
        }
    }
}

pub trait DensityProfile {
    fn density(&self, pos: VecLength) -> Density;
    fn max_value(&self) -> Density;
}

pub struct ConstantDensity(pub Density);

impl DensityProfile for ConstantDensity {
    fn density(&self, _pos: VecLength) -> Density {
        self.0
    }
    fn max_value(&self) -> Density {
        self.0
    }
}

pub struct Sample {
    positions: Vec<VecLength>,
    mass_per_particle: Mass,
}

impl Sample {
    pub fn spawn_with(
        self,
        commands: &mut Commands,
        spawn_additional_components: impl Fn(&mut EntityCommands, VecLength, Mass),
    ) {
        for pos in self.positions.into_iter() {
            let mut entity_commands = commands.spawn_bundle((
                LocalParticle,
                Position(pos),
                components::Mass(self.mass_per_particle),
            ));
            spawn_additional_components(&mut entity_commands, pos, self.mass_per_particle);
        }
    }

    pub fn spawn_with_zero_velocity(self, commands: &mut Commands) {
        self.spawn_with(commands, |entity_commands, _, _| {
            entity_commands.insert(components::Velocity(VecVelocity::zero()));
        });
    }
}

pub struct Sampler<P: DensityProfile> {
    profile: P,
    extent: Extent,
    num_samples: usize,
    resolution_spec: ResolutionSpecification,
    rng: StdRng,
}

impl<P: DensityProfile> Sampler<P> {
    pub fn new(profile: P, extent: Extent, resolution_spec: ResolutionSpecification) -> Self {
        Self {
            profile,
            extent,
            resolution_spec,
            num_samples: 100000,
            rng: StdRng::seed_from_u64(DEFAULT_SEED),
        }
    }

    pub fn sample(mut self) -> Sample {
        let total_mass_profile = self.integrate();
        let volume = self.extent.volume();
        let num_particles_specified =
            (self.resolution_spec.as_number_density(volume) * volume).value() as usize;
        let mass_per_particle = total_mass_profile / num_particles_specified as Float;
        let mut positions = vec![];
        // A simple implementation of rejection sampling
        while positions.len() < num_particles_specified {
            let pos = gen_range(&mut self.rng, self.extent.min, self.extent.max);
            let random_density = self
                .rng
                .gen_range(Density::zero()..self.profile.max_value());
            if random_density < self.profile.density(pos) {
                positions.push(pos);
            }
        }
        Sample {
            positions,
            mass_per_particle,
        }
    }

    fn integrate(&mut self) -> Mass {
        let volume_per_sample = self.extent.volume() / (self.num_samples as Float);
        let mut mass = Mass::zero();
        for _ in 0..self.num_samples {
            let pos = gen_range(&mut self.rng, self.extent.min, self.extent.max);
            mass += self.profile.density(pos) * volume_per_sample;
        }
        mass
    }
}
