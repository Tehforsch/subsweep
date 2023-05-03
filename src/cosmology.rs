use derive_custom::raxiom_parameters;

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
            Cosmology::Cosmological { a, h } => Dimensionless::dimensionless(*a),
            Cosmology::NonCosmological => Dimensionless::dimensionless(1.0),
        }
    }
}
