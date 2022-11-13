mod density_profile;
mod monte_carlo_sampler;
mod regular;
mod resolution;
mod velocity_profile;

use bevy::prelude::Commands;
pub use regular::IntegerTuple;
pub use regular::RegularSampler;

pub use self::density_profile::ConstantDensity;
pub use self::density_profile::DensityProfile;
pub use self::monte_carlo_sampler::MonteCarloSampler;
pub use self::resolution::Resolution;
pub use self::velocity_profile::ConstantVelocity;
pub use self::velocity_profile::VelocityProfile;
pub use self::velocity_profile::ZeroVelocity;
use crate::components;
use crate::parameters::SimulationBox;
use crate::prelude::LocalParticle;
use crate::prelude::Named;
use crate::simulation::RaxiomPlugin;
use crate::units::Density;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecVelocity;

pub const DEFAULT_SEED: u64 = 123;

pub struct PreSample {
    positions: Vec<VecLength>,
    masses: Vec<Mass>,
}

pub struct Sample {
    positions: Vec<VecLength>,
    velocities: Vec<VecVelocity>,
    masses: Vec<Mass>,
}

impl Sample {
    fn new(pre_sample: PreSample, velocity_profile: &dyn VelocityProfile) -> Self {
        let velocities = pre_sample
            .positions
            .iter()
            .map(|pos| velocity_profile.velocity(*pos))
            .collect();
        Self {
            positions: pre_sample.positions,
            velocities,
            masses: pre_sample.masses,
        }
    }

    fn spawn(&mut self, commands: &mut Commands) {
        assert_eq!(self.positions.len(), self.velocities.len());
        for (pos, (mass, vel)) in self
            .positions
            .drain(..)
            .zip(self.masses.drain(..).zip(self.velocities.drain(..)))
        {
            commands.spawn((
                LocalParticle,
                components::Position(pos),
                components::Mass(mass),
                components::Velocity(vel),
            ));
        }
    }
}

pub struct SamplingData {
    density_profile: Box<dyn DensityProfile>,
    box_: SimulationBox,
}

pub trait Sampler {
    fn sample(&self, data: &SamplingData) -> PreSample;
}

#[derive(Named)]
pub struct InitialConditionsPlugin {
    density_profile: Box<dyn DensityProfile>,
    velocity_profile: Box<dyn VelocityProfile>,
    sampler: Box<dyn Sampler>,
}

impl Default for InitialConditionsPlugin {
    fn default() -> Self {
        Self {
            density_profile: Box::new(ConstantDensity(Density::zero())),
            velocity_profile: Box::new(ZeroVelocity),
            sampler: Box::new(MonteCarloSampler::num_particles(100)),
        }
    }
}

impl InitialConditionsPlugin {
    pub fn density_profile(mut self, density_profile: impl DensityProfile + 'static) -> Self {
        self.density_profile = Box::new(density_profile);
        self
    }

    pub fn velocity_profile(mut self, velocity_profile: impl VelocityProfile + 'static) -> Self {
        self.velocity_profile = Box::new(velocity_profile);
        self
    }

    pub fn sampler(mut self, sampler: impl Sampler + 'static) -> Self {
        self.sampler = Box::new(sampler);
        self
    }
}

impl RaxiomPlugin for InitialConditionsPlugin {
    fn build_on_main_rank(&self, sim: &mut crate::simulation::Simulation) {
        let box_ = sim.get_parameters::<SimulationBox>();
        let data = SamplingData {
            density_profile: self.density_profile.clone_box(),
            box_: box_.clone(),
        };
        let mut sample = Sample::new(self.sampler.sample(&data), &*self.velocity_profile);
        sim.add_startup_system(move |commands: Commands| {
            initial_conditions_system(commands, &mut sample)
        });
    }
}

fn initial_conditions_system(mut commands: Commands, sample: &mut Sample) {
    sample.spawn(&mut commands);
}
