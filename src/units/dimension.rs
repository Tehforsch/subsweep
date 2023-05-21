use diman::diman_dimension;

const MASS_TO_SI: f64 = 1.0;
const LENGTH_TO_SI: f64 = 1.0;
const TIME_TO_SI: f64 = 1.0;
const TEMPERATURE_TO_SI: f64 = 1.0;

#[derive(PartialEq, Eq, Debug, Clone)]
#[diman_dimension]
pub struct Dimension {
    pub length: i32,
    pub time: i32,
    pub mass: i32,
    pub temperature: i32,
    pub h: i32,
    pub a: i32,
}

pub const NONE: Dimension = Dimension {
    length: 0,
    time: 0,
    mass: 0,
    temperature: 0,
    h: 0,
    a: 0,
};

impl Dimension {
    /// Get the base conversion factor of this dimension
    /// into SI units. As of now, this is always 1.0
    /// but will change if the base units are changed
    /// from SI to anything else
    pub fn base_conversion_factor(&self) -> f64 {
        // Make sure we notice this whenever we add a field to Dimension:
        let Self {
            length,
            time,
            mass,
            temperature,
            h: _,
            a: _,
        } = self;
        (LENGTH_TO_SI).powi(*length)
            * (TIME_TO_SI).powi(*time)
            * (MASS_TO_SI).powi(*mass)
            * (TEMPERATURE_TO_SI).powi(*temperature)
    }
}
