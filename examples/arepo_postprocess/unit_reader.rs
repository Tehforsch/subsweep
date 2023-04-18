use hdf5::Dataset;
use raxiom::io::UnitReader;
use raxiom::units::Dimension;

use crate::cosmology::Cosmology;

pub const SCALE_FACTOR_IDENTIFIER: &str = "to_cgs";
pub const LENGTH_IDENTIFIER: &str = "length_scaling";
pub const VELOCITY_IDENTIFIER: &str = "velocity_scaling";
pub const MASS_IDENTIFIER: &str = "mass_scaling";
pub const A_IDENTIFIER: &str = "a_scaling";
pub const H_IDENTIFIER: &str = "h_scaling";

#[derive(Clone)]
pub struct ArepoUnitReader {
    cosmology: Cosmology,
}

impl ArepoUnitReader {
    pub fn new(cosmology: Cosmology) -> Self {
        Self { cosmology }
    }

    fn get_cosmological_scale_factor(&self, dimension: &Dimension) -> f64 {
        self.check_cosmology_available();
        match self.cosmology {
            Cosmology::Cosmological { a, h } => a.powi(dimension.a) * h.powi(dimension.h),
            Cosmology::NonCosmological => unreachable!(),
        }
    }

    fn check_cosmology_available(&self) {
        match self.cosmology {
            Cosmology::Cosmological { .. } => {},
            Cosmology::NonCosmological => panic!("Cosmological units in input file, but no cosmology given. Add cosmology section to parameter file?"),
        }
    }
}

fn is_cosmological(dimension: &Dimension) -> bool {
    dimension.a != 0 || dimension.h != 0
}

impl ArepoUnitReader {
    fn read_raw_dimension(&self, set: &Dataset) -> Dimension {
        let read_attr =
            |ident, error_message| set.attr(ident).expect(error_message).read_scalar().unwrap();
        let length: i32 = read_attr(LENGTH_IDENTIFIER, "No length scale factor in dataset");
        let mass: i32 = read_attr(MASS_IDENTIFIER, "No mass scale factor in dataset");
        let velocity: i32 = read_attr(VELOCITY_IDENTIFIER, "No time scale factor in dataset");
        let a: i32 = read_attr(A_IDENTIFIER, "No a scale factor in dataset");
        let h: i32 = read_attr(H_IDENTIFIER, "No h scale factor in dataset");
        let length = length + velocity;
        let time = -velocity;

        Dimension {
            length,
            mass,
            time,
            temperature: 0,
            amount: 0,
            h,
            a,
        }
    }
}

impl UnitReader for ArepoUnitReader {
    fn read_scale_factor(&self, set: &Dataset) -> f64 {
        let dimension = self.read_raw_dimension(set);
        let cosmology_scale_factor = if is_cosmological(&dimension) {
            self.get_cosmological_scale_factor(&dimension)
        } else {
            1.0
        };
        let cgs_to_si = 0.01f64.powi(dimension.length) * 0.001f64.powi(dimension.mass);
        let cgs: f64 = set
            .attr(SCALE_FACTOR_IDENTIFIER)
            .expect("No scale factor in dataset")
            .read_scalar()
            .unwrap();
        cosmology_scale_factor * cgs_to_si * cgs
    }

    fn read_dimension(&self, set: &Dataset) -> Dimension {
        let dimension = self.read_raw_dimension(set);
        if !is_cosmological(&dimension) {
            dimension
        } else {
            self.check_cosmology_available();
            Dimension {
                a: 0,
                h: 0,
                ..dimension
            }
        }
    }
}
