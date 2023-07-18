use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::Resource;
use derive_custom::raxiom_parameters;
use derive_custom::Named;
use hdf5::H5Type;

use crate::impl_attribute;
use crate::io::output::ToAttribute;
use crate::units::Dimension;
use crate::units::Dimensionless;
use crate::units::Time;

#[raxiom_parameters("cosmology")]
#[derive(Copy, Named, Debug)]
#[serde(untagged)]
pub enum Cosmology {
    Cosmological {
        a: f64,
        h: f64,
        params: Option<CosmologyParams>,
    },
    NonCosmological,
}

#[raxiom_parameters]
#[derive(Copy, Named, Debug)]
pub struct CosmologyParams {
    omega_0: f64,
    omega_lambda: f64,
}

impl Cosmology {
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
        match self {
            Cosmology::Cosmological { a, h, .. } => a.powi(dimension.a) * h.powi(dimension.h),
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
}

pub fn set_cosmology_attributes_system(mut commands: Commands, cosmology: Res<Cosmology>) {
    commands.insert_resource(ScaleFactor(cosmology.scale_factor()));
    commands.insert_resource(LittleH(cosmology.little_h()));
}

#[derive(H5Type, Clone, Copy, Named, Resource)]
#[repr(transparent)]
#[name = "scale_factor"]
pub struct ScaleFactor(pub Dimensionless);

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
impl_attribute!(LittleH, Dimensionless);
impl_attribute!(OmegaLambda, Dimensionless);
impl_attribute!(Omega0, Dimensionless);

#[cfg(test)]
mod tests {
    use super::CosmologyParams;
    use crate::units::Time;

    #[test]
    fn time_difference_between_scalefactors() {
        let cosmology = CosmologyParams {
            omega_lambda: 0.6911,
            omega_0: 0.308983,
        };
        let diff = |a0: f64, a1: f64| {
            cosmology.time_difference_between_scalefactors(a0.into(), a1.into(), 0.6774.into())
        };
        assert_eq!(diff(1.0, 1.0), Time::zero());
        assert!((diff(0.99, 1.0) - Time::gigayears(0.14473176)).abs() < Time::years(10000.0));
    }
}
