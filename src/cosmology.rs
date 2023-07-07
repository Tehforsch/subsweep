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

#[raxiom_parameters("cosmology")]
#[derive(Copy, Named, Debug)]
#[serde(untagged)]
pub enum Cosmology {
    Cosmological { a: f64, h: f64 },
    NonCosmological,
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
            Cosmology::Cosmological { a, h } => a.powi(dimension.a) * h.powi(dimension.h),
            Cosmology::NonCosmological => panic!("Tried to convert cosmological units without cosmology. Add cosmology section to parameter file?"),
        }
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

impl_attribute!(ScaleFactor, Dimensionless);
impl_attribute!(LittleH, Dimensionless);
