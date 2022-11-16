use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub use super::resolution::Resolution;
use super::PreSample;
use super::Sampler;
use super::SamplingData;
use crate::prelude::Float;
use crate::rand::gen_range;
use crate::units::Density;
use crate::units::Mass;

pub const DEFAULT_SEED: u64 = 123;

pub struct MonteCarloSampler {
    pub num_samples: usize,
    pub seed: u64,
    pub resolution: Resolution,
}

impl MonteCarloSampler {
    pub fn num_particles(num_particles: usize) -> Self {
        Self {
            num_samples: 100000,
            seed: DEFAULT_SEED,
            resolution: Resolution::NumParticles(num_particles),
        }
    }
}

impl MonteCarloSampler {
    fn integrate(&self, rng: &mut StdRng, data: &SamplingData) -> Mass {
        let volume_per_sample = data.box_.volume() / (self.num_samples as Float);
        let mut mass = Mass::zero();
        for _ in 0..self.num_samples {
            let pos = gen_range(rng, data.box_.min, data.box_.max);
            mass += data.density_profile.density(&data.box_, pos) * volume_per_sample;
        }
        mass
    }
}

impl Sampler for MonteCarloSampler {
    fn sample(&self, data: &SamplingData) -> PreSample {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let total_mass_profile = self.integrate(&mut rng, data);
        let volume = data.box_.volume();
        let num_particles_specified =
            (self.resolution.as_number_density(volume) * volume).value() as usize;
        let mass_per_particle = total_mass_profile / num_particles_specified as Float;
        let mut positions = vec![];
        // A simple implementation of rejection sampling
        while positions.len() < num_particles_specified {
            let pos = gen_range(&mut rng, data.box_.min, data.box_.max);
            let random_density = rng.gen_range(Density::zero()..data.density_profile.max_value());
            if random_density < data.density_profile.density(&data.box_, pos) {
                positions.push(pos);
            }
        }
        let masses = positions.iter().map(|_| mass_per_particle).collect();
        PreSample { positions, masses }
    }
}
