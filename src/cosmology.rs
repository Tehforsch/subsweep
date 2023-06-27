use derive_custom::raxiom_parameters;

use crate::units::Dimension;
use crate::units::Dimensionless;

#[raxiom_parameters("cosmology")]
#[derive(Debug)]
#[serde(untagged)]
pub enum Cosmology {
    Cosmological { a: f64, h: f64 },
    NonCosmological,
}

impl Cosmology {
    pub fn scale_factor(&self) -> Dimensionless {
        match self {
            Cosmology::Cosmological { a, h: _ } => Dimensionless::dimensionless(*a),
            Cosmology::NonCosmological => Dimensionless::dimensionless(1.0),
        }
    }

    pub fn get_factor(&self, dimension: &Dimension) -> f64 {
        match self {
            Cosmology::Cosmological { a, h } => a.powi(dimension.a) * h.powi(dimension.h),
            Cosmology::NonCosmological => panic!("Tried to convert cosmological units without cosmology. Add cosmology section to parameter file?"),
        }
    }
}
