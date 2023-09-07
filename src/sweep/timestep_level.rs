use std::ops::Add;
use std::ops::SubAssign;

use mpi::traits::Equivalence;

use crate::units::helpers::Float;
use crate::units::Time;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Equivalence, Hash)]
pub struct TimestepLevel(pub usize);

impl Add<usize> for TimestepLevel {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl SubAssign<usize> for TimestepLevel {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
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

    pub fn is_active(&self, current_level: TimestepLevel) -> bool {
        *self >= current_level
    }

    pub fn to_timestep(&self, max_timestep: Time) -> Time {
        max_timestep * self.as_factor()
    }

    pub fn as_factor(&self) -> f64 {
        (0.5 as Float).powi(self.0 as i32)
    }

    pub fn is_highest_timestep(&self) -> bool {
        self.0 == 0
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
