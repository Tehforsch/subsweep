use super::timestep_level::TimestepLevel;
use crate::units::Time;

#[derive(Clone, Copy)]
pub struct TimesteppingState {
    max_timestep: Time,
    max_num_timestep_levels: usize,
    current_lowest_allowed: TimestepLevel,
}

impl TimesteppingState {
    pub fn new(max_timestep: Time, max_num_timestep_levels: usize) -> Self {
        Self {
            max_timestep,
            max_num_timestep_levels,
            current_lowest_allowed: TimestepLevel(1),
        }
    }

    pub fn iter_levels_in_sweep_order(self) -> impl Iterator<Item = TimestepLevel> {
        let num = self.num_allowed_levels();
        (0..(2usize.pow(num as u32 - 1))).map(move |i| {
            self.lowest_active_from_iteration((self.current_lowest_allowed + i).0 as u32)
        })
    }

    pub fn iter_allowed_levels(self) -> impl Iterator<Item = TimestepLevel> {
        (self.current_lowest_allowed.0..self.max_num_timestep_levels)
            .map(move |level| TimestepLevel(level))
    }

    pub fn timestep_at_level(self, level: TimestepLevel) -> Time {
        level.to_timestep(self.max_timestep)
    }

    pub fn get_desired_level_from_desired_timestep(self, desired_timestep: Time) -> TimestepLevel {
        let mut level = TimestepLevel::from_max_timestep_and_desired_timestep(
            self.max_num_timestep_levels,
            self.max_timestep,
            desired_timestep,
        );
        if level < self.current_lowest_allowed {
            level = self.current_lowest_allowed;
        }
        level
    }

    fn num_allowed_levels(&self) -> usize {
        self.max_num_timestep_levels - self.current_lowest_allowed.0
    }

    fn lowest_active_from_iteration(&self, iteration: u32) -> TimestepLevel {
        // find the lowest set bit in iteration.
        assert!(self.max_num_timestep_levels < 32);
        let first_bit = find_index_of_lowest_set_bit_in_int(iteration)
            .unwrap_or(self.max_num_timestep_levels as u32 - 1);
        TimestepLevel(self.max_num_timestep_levels - 1 - first_bit as usize)
    }
}

fn find_index_of_lowest_set_bit_in_int(iteration: u32) -> Option<u32> {
    for bit_num in 0..32 {
        let mask = 1 << bit_num;
        if iteration & mask > 0 {
            return Some(bit_num);
        }
    }
    None
}
