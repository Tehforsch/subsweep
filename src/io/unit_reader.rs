use hdf5::Dataset;

use super::output::LENGTH_IDENTIFIER;
use super::output::MASS_IDENTIFIER;
use super::output::SCALE_FACTOR_IDENTIFIER;
use super::output::TEMPERATURE_IDENTIFIER;
use super::output::TIME_IDENTIFIER;
use crate::units::Dimension;
use crate::units::NONE;

pub trait UnitReaderClone {
    fn clone_box(&self) -> Box<dyn UnitReader>;
}

pub trait UnitReader: UnitReaderClone {
    fn read_scale_factor(&self, set: &Dataset) -> f64;
    fn read_dimension(&self, set: &Dataset) -> Dimension;
}

impl<T> UnitReaderClone for T
where
    T: 'static + Clone + UnitReader,
{
    fn clone_box(&self) -> Box<dyn UnitReader> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn UnitReader> {
    fn clone(&self) -> Box<dyn UnitReader> {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct DefaultUnitReader;

impl UnitReader for DefaultUnitReader {
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
        let time: i32 = read_attr(TIME_IDENTIFIER, "No time scale factor in dataset");
        let h = 0;
        let a = 0;
        let temperature: i32 = read_attr(
            TEMPERATURE_IDENTIFIER,
            "No temperature scale factor in dataset",
        );
        Dimension {
            length,
            mass,
            time,
            temperature,
            h,
            a,
        }
    }
}

#[derive(Clone)]
pub struct IdReader;

impl UnitReader for IdReader {
    fn read_scale_factor(&self, _set: &Dataset) -> f64 {
        1.0
    }

    fn read_dimension(&self, _set: &Dataset) -> Dimension {
        NONE
    }
}
