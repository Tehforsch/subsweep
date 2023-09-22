use bevy_ecs::prelude::Commands;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::Resource;
use derive_custom::subsweep_parameters;
use derive_custom::Named;
use hdf5::H5Type;

use crate::impl_attribute;
use crate::io::output::ToAttribute;
use crate::units::Dimension;
use crate::units::Dimensionless;
use crate::units::Time;

#[subsweep_parameters("cosmology")]
#[derive(Named, Debug)]
#[serde(untagged)]
pub enum Cosmology {
    Cosmological {
        a: f64,
        h: f64,
        params: Option<CosmologyParams>,
    },
    NonCosmological,
}

#[subsweep_parameters]
#[derive(Copy, Named, Debug)]
pub struct CosmologyParams {
    omega_0: f64,
    omega_lambda: f64,
}

pub fn scalefactor_to_redshift(a: Dimensionless) -> Dimensionless {
    1.0 / a - 1.0
}

impl Cosmology {
    pub fn redshift(&self) -> Dimensionless {
        scalefactor_to_redshift(self.scale_factor())
    }

    pub fn scale_factor(&self) -> Dimensionless {
        match self {
            Cosmology::Cosmological { a, .. } => Dimensionless::dimensionless(*a),
            Cosmology::NonCosmological => Dimensionless::dimensionless(1.0),
        }
    }

    pub fn little_h(&self) -> Dimensionless {
        match self {
            Cosmology::Cosmological { h, .. } => Dimensionless::dimensionless(*h),
            Cosmology::NonCosmological => panic!("Tried to get little h without cosmology."),
        }
    }

    pub fn get_factor(&self, dimension: &Dimension) -> f64 {
        // h is fucking stupid and doesnt follow the rules
        match self {
            Cosmology::Cosmological { a, h, .. } => a.powi(-dimension.a) * h.powi(dimension.h),
            Cosmology::NonCosmological => panic!("Tried to convert cosmological units without cosmology. Add cosmology section to parameter file?"),
        }
    }

    pub fn time_difference_between_scalefactors(
        &self,
        a0: Dimensionless,
        a1: Dimensionless,
    ) -> Time {
        match self {
            Cosmology::Cosmological { h, params, .. } => params
                .unwrap()
                .time_difference_between_scalefactors(a0, a1, Dimensionless::dimensionless(*h)),
            Cosmology::NonCosmological => {
                panic!("Tried to compute time difference in non cosmological run")
            }
        }
    }
}

impl CosmologyParams {
    pub fn time_difference_between_scalefactors(
        &self,
        a0: Dimensionless,
        a1: Dimensionless,
        h: Dimensionless,
    ) -> Time {
        const HUBBLE: f64 = 3.2407789e-18; /* in h/sec */
        let Self {
            omega_lambda,
            omega_0,
            ..
        } = self;
        if ((omega_lambda + omega_0) - 1.0).abs() > 1e-2 {
            unimplemented!("Cosmology needs to be flat.");
        }

        let time = |a: Dimensionless| {
            let factor1 = 2.0 / (3.0 * omega_lambda.sqrt());
            let term1 = (omega_lambda / omega_0).sqrt() * a.powf(1.5);
            let term2 = (1.0 + omega_lambda / omega_0 * a.cubed()).sqrt();
            let factor2 = (term1 + term2).ln();
            factor1 * factor2
        };

        let t0 = time(a0);
        let t1 = time(a1);
        Time::seconds(*(t1 - t0) / (HUBBLE * *h))
    }

    /// Get the scale factor a which the given cosmology has when
    /// delta_t time elapses after the time at which the scale factor was a.
    pub fn get_scalefactor_from_scalefactor_and_time_difference(
        &self,
        a: Dimensionless,
        h: Dimensionless,
        delta_t: Time,
    ) -> Dimensionless {
        let min = 0.0;
        let max = 1.0;
        binary_search(
            |a1| {
                (self.time_difference_between_scalefactors(a, a1.into(), h) - delta_t)
                    .value_unchecked()
            },
            min,
            max,
            Time::years(100.0).value_unchecked(),
        )
        .into()
    }
}

/// Find a root of the monotonously increasing function f by binary search on the interval [min, max].
fn binary_search(f: impl Fn(f64) -> f64, min: f64, max: f64, threshold: f64) -> f64 {
    depth_limited_binary_search(f, min, max, threshold, 0)
}

fn depth_limited_binary_search(
    f: impl Fn(f64) -> f64,
    min: f64,
    max: f64,
    threshold: f64,
    depth: usize,
) -> f64 {
    const MAX_DEPTH: usize = 100;
    if depth > MAX_DEPTH {
        panic!("Binary search failed");
    }
    let guess = (min + max) / 2.0;
    let val = f(guess);
    if val.abs() <= threshold {
        guess
    } else {
        if val.is_sign_negative() {
            depth_limited_binary_search(f, guess, max, threshold, depth + 1)
        } else {
            depth_limited_binary_search(f, min, guess, threshold, depth + 1)
        }
    }
}

pub fn set_initial_cosmology_attributes_system(mut commands: Commands, cosmology: Res<Cosmology>) {
    commands.insert_resource(ScaleFactor(cosmology.scale_factor()));
    commands.insert_resource(Redshift(cosmology.redshift()));
    commands.insert_resource(LittleH(cosmology.little_h()));
}

#[derive(H5Type, Clone, Copy, Named, Resource)]
#[repr(transparent)]
#[name = "scale_factor"]
pub struct ScaleFactor(pub Dimensionless);

#[derive(H5Type, Clone, Copy, Named, Resource)]
#[repr(transparent)]
#[name = "redshift"]
pub struct Redshift(pub Dimensionless);

#[derive(H5Type, Clone, Copy, Named, Resource)]
#[repr(transparent)]
#[name = "little_h"]
pub struct LittleH(pub Dimensionless);

#[derive(H5Type, Clone, Copy, Named, Resource)]
#[repr(transparent)]
#[name = "omega_lambda"]
pub struct OmegaLambda(pub Dimensionless);

#[derive(H5Type, Clone, Copy, Named, Resource)]
#[repr(transparent)]
#[name = "omega_0"]
pub struct Omega0(pub Dimensionless);

impl_attribute!(ScaleFactor, Dimensionless);
impl_attribute!(Redshift, Dimensionless);
impl_attribute!(LittleH, Dimensionless);
impl_attribute!(OmegaLambda, Dimensionless);
impl_attribute!(Omega0, Dimensionless);

#[cfg(test)]
mod tests {
    use super::CosmologyParams;
    use crate::units::Dimensionless;
    use crate::units::Time;

    fn get_test_cosmology_and_h() -> (CosmologyParams, Dimensionless) {
        let cosmology = CosmologyParams {
            omega_lambda: 0.6911,
            omega_0: 0.308983,
        };
        (cosmology, 0.6774.into())
    }

    #[test]
    fn time_difference_between_scalefactors() {
        let (cosmology, h) = get_test_cosmology_and_h();
        let diff = |a0: f64, a1: f64| {
            cosmology.time_difference_between_scalefactors(a0.into(), a1.into(), h)
        };
        assert_eq!(diff(1.0, 1.0), Time::zero());
        assert!((diff(0.99, 1.0) - Time::gigayears(0.14473176)).abs() < Time::years(10000.0));
    }

    #[test]
    fn get_scalefactor_from_scalefactor_and_time_difference() {
        let (cosmology, h) = get_test_cosmology_and_h();
        for a0 in [0.01, 0.1, 0.2, 0.5, 0.9] {
            for a1 in [0.01, 0.1, 0.2, 0.5, 0.9] {
                let delta_t =
                    cosmology.time_difference_between_scalefactors(a0.into(), a1.into(), h);
                let diff = (cosmology.get_scalefactor_from_scalefactor_and_time_difference(
                    a0.into(),
                    h,
                    delta_t,
                ) - a1)
                    .abs();
                assert!(diff < 1e-5);
            }
        }
    }
}
