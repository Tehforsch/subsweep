use hdf5::Dataset;
use raxiom::io::UnitReader;
use raxiom::units::Dimension;

pub const SCALE_FACTOR_IDENTIFIER: &str = "to_cgs";
pub const LENGTH_IDENTIFIER: &str = "length_scaling";
pub const VELOCITY_IDENTIFIER: &str = "velocity_scaling";
pub const MASS_IDENTIFIER: &str = "mass_scaling";
pub const A_IDENTIFIER: &str = "a_scaling";
pub const H_IDENTIFIER: &str = "h_scaling";

#[derive(Clone)]
pub struct ArepoUnitReader;

impl UnitReader for ArepoUnitReader {
    fn read_scale_factor(&self, set: &Dataset) -> f64 {
        set.attr(SCALE_FACTOR_IDENTIFIER)
            .expect("No scale factor in dataset")
            .read_scalar()
            .unwrap()
    }

    fn read_dimension(&self, set: &Dataset) -> Dimension {
        let read_attr =
            |ident, error_message| set.attr(ident).expect(error_message).read_scalar().unwrap();
        let length: i32 = read_attr(LENGTH_IDENTIFIER, "No length scale factor in dataset");
        let mass: i32 = read_attr(MASS_IDENTIFIER, "No mass scale factor in dataset");
        let velocity: i32 = read_attr(VELOCITY_IDENTIFIER, "No time scale factor in dataset");
        let a: i32 = read_attr(A_IDENTIFIER, "No a scale factor in dataset");
        let h: i32 = read_attr(H_IDENTIFIER, "No h scale factor in dataset");
        assert_eq!(
            a, 0,
            "Tried to read dataset with a_scaling != 0. Cosmological units not implemented yet."
        );
        assert_eq!(
            h, 0,
            "Tried to read dataset with h_scaling != 0. Cosmological units not implemented yet."
        );
        let length = length + velocity;
        let time = -velocity;

        Dimension {
            length,
            mass,
            time,
            temperature: 0,
            amount: 0,
        }
    }
}
