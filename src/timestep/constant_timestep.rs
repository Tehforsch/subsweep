use super::TimestepCriterion;
use super::TimestepParameters;
use crate::units::Time;

pub struct ConstantTimestep;

impl TimestepCriterion for ConstantTimestep {
    type Filter = ();

    type Query = ();

    fn timestep(parameters: &TimestepParameters, _query_item: ()) -> Time {
        parameters.max_timestep
    }
}
