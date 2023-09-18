use log::error;
use serde::Serialize;

use crate::cosmology::scalefactor_to_redshift;
use crate::parameters::Cosmology;
use crate::units::Dimensionless;
use crate::units::Time;

#[derive(Serialize)]
#[serde(untagged)]
pub enum TimeSpec {
    Time(Time),
    Cosmological(CosmologicalTime),
}

impl TimeSpec {
    pub fn new(time: Time, cosmology: &Cosmology) -> Self {
        match cosmology {
            Cosmology::Cosmological { a, h, params } => {
                if let Some(params) = params {
                    TimeSpec::Cosmological(CosmologicalTime::new(
                        time,
                        params.get_scalefactor_from_scalefactor_and_time_difference(
                            (*a).into(),
                            (*h).into(),
                            time,
                        ),
                    ))
                } else {
                    error!("No cosmological parameters given. Cannot determine current redshift and scale factor for output.");
                    TimeSpec::Time(time)
                }
            }
            Cosmology::NonCosmological => TimeSpec::Time(time),
        }
    }
}

#[derive(Serialize)]
pub struct CosmologicalTime {
    time_elapsed: Time,
    pub redshift: Dimensionless,
    pub scale_factor: Dimensionless,
}

impl CosmologicalTime {
    fn new(time_elapsed: Time, scale_factor: Dimensionless) -> Self {
        Self {
            time_elapsed,
            scale_factor,
            redshift: scalefactor_to_redshift(scale_factor),
        }
    }
}
