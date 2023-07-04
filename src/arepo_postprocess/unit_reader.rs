use bevy::prelude::debug;
use hdf5::Dataset;
use raxiom::components::Position;
use raxiom::cosmology::Cosmology;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::io::UnitReader;
use raxiom::prelude::Float;
use raxiom::prelude::MVec;
use raxiom::units::Dimension;
use raxiom::units::VecLength;
use raxiom::units::NONE;

pub const SCALE_FACTOR_IDENTIFIER: &str = "to_cgs";
pub const LENGTH_IDENTIFIER: &str = "length_scaling";
pub const VELOCITY_IDENTIFIER: &str = "velocity_scaling";
pub const MASS_IDENTIFIER: &str = "mass_scaling";
pub const A_IDENTIFIER: &str = "a_scaling";
pub const H_IDENTIFIER: &str = "h_scaling";

/// This special reader exists arepo writes out different vector types
/// than raxiom.
pub fn read_vec(data: &[Float]) -> Position {
    Position(VecLength::new_unchecked(MVec::new(
        data[0], data[1], data[2],
    )))
}

pub fn make_descriptor<T, U: UnitReader + Clone + 'static>(
    unit_reader: &U,
    name: &str,
    shape: DatasetShape<T>,
) -> InputDatasetDescriptor<T> {
    InputDatasetDescriptor::<T> {
        descriptor: DatasetDescriptor {
            dataset_name: name.into(),
            unit_reader: Box::new(unit_reader.clone()),
        },
        shape,
    }
}

#[derive(Clone)]
pub struct ArepoUnitReader {
    cosmology: Cosmology,
}

impl ArepoUnitReader {
    pub fn new(cosmology: Cosmology) -> Self {
        Self { cosmology }
    }

    fn get_cosmological_factor(&self, dimension: &Dimension) -> f64 {
        self.check_cosmology_available();
        self.cosmology.get_factor(dimension)
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
        let read_attr = |ident, name| {
            set.attr(ident)
                .unwrap_or_else(|_| panic!("No {} in dataset: '{}'", name, set.name()))
                .read_scalar()
                .unwrap()
        };
        let length: i32 = read_attr(LENGTH_IDENTIFIER, "length scale factor");
        let mass: i32 = read_attr(MASS_IDENTIFIER, "mass scale factor");
        let velocity: i32 = read_attr(VELOCITY_IDENTIFIER, "velocity scale factor");
        let a: i32 = read_attr(A_IDENTIFIER, "a scale factor");
        let h: i32 = read_attr(H_IDENTIFIER, "h scale factor");
        let length = length + velocity;
        let time = -velocity;

        Dimension {
            length,
            mass,
            time,
            temperature: 0,
            h,
            a,
        }
    }
}

impl UnitReader for ArepoUnitReader {
    fn read_scale_factor(&self, set: &Dataset) -> f64 {
        let dimension = self.read_raw_dimension(set);
        let cosmology_scale_factor = if is_cosmological(&dimension) {
            self.get_cosmological_factor(&dimension)
        } else {
            1.0
        };
        let cgs_to_si = 0.01f64.powi(dimension.length) * 0.001f64.powi(dimension.mass);
        let mut cgs: f64 = set
            .attr(SCALE_FACTOR_IDENTIFIER)
            .expect("No scale factor in dataset")
            .read_scalar()
            .unwrap();
        if cgs == 0.0 {
            assert_eq!(dimension, NONE);
            debug!("Unit scale factor is 0 in dimensionless dataset. Assuming 1.");
            cgs = 1.0;
        }
        cosmology_scale_factor * cgs_to_si * cgs
    }

    fn read_dimension(&self, set: &Dataset) -> Dimension {
        let dimension = self.read_raw_dimension(set);
        if !is_cosmological(&dimension) {
            dimension
        } else {
            self.check_cosmology_available();
            dimension.non_cosmological()
        }
    }
}
