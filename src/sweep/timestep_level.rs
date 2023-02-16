use bevy::prelude::Component;
use mpi::traits::Equivalence;

use crate::units::helpers::Float;
use crate::units::Time;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Component, Equivalence, Hash)]
pub struct TimestepLevel(pub usize);

fn find_index_of_lowest_set_bit_in_int(iteration: u32) -> Option<u32> {
    for bit_num in 0..32 {
        let mask = 1 << bit_num;
        if iteration & mask > 0 {
            return Some(bit_num);
        }
    }
    None
}

impl TimestepLevel {
    pub fn from_max_timestep_and_desired_timestep(
        max_num_levels: usize,
        max_timestep: Time,
        desired_timestep: Time,
    ) -> Self {
        let ratio = max_timestep / desired_timestep;
        let level = ratio.log2().ceil().value() as usize;
        let result = level.clamp(0, max_num_levels - 1);
        Self(result)
    }

    pub fn lowest_active_from_iteration(num_levels: usize, iteration: u32) -> Self {
        // find the lowest set bit in iteration.
        assert!(num_levels < 32);
        let first_bit =
            find_index_of_lowest_set_bit_in_int(iteration).unwrap_or(num_levels as u32 - 1);
        Self(num_levels - 1 - first_bit as usize)
    }

    pub fn is_active(&self, current_level: TimestepLevel) -> bool {
        *self >= current_level
    }

    pub fn to_timestep(&self, max_timestep: Time) -> Time {
        max_timestep / (2.0 as Float).powi(self.0 as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::TimestepLevel;
    use crate::units::Time;

    #[test]
    fn compute_timestep_level() {
        let check_level = |max_num_levels, secs_desired, result| {
            println!("{} {} {}", max_num_levels, secs_desired, result);
            assert_eq!(
                TimestepLevel::from_max_timestep_and_desired_timestep(
                    max_num_levels,
                    Time::seconds(1.0),
                    Time::seconds(secs_desired)
                ),
                TimestepLevel(result)
            );
        };
        check_level(1, 1.0, 0);
        check_level(2, 1.0, 0);
        check_level(1, 0.001, 0);
        check_level(2, 0.001, 1);
        check_level(3, 0.001, 2);
        check_level(2, 0.500001, 1);
        check_level(2, 0.499999, 1);
        check_level(3, 0.499999, 2);
        check_level(5, 100.0, 0);
        check_level(5, 0.0, 4);
    }
}
