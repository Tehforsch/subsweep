use super::timestep_level::TimestepLevel;
use crate::units::Time;

#[derive(Clone, Copy)]
pub struct TimestepState {
    max_timestep: Time,
    max_num_timestep_levels: usize,
    current_lowest_allowed: TimestepLevel,
    first_update_at_highest_level_done: bool,
}

impl TimestepState {
    pub fn new(max_timestep: Time, max_num_timestep_levels: usize) -> Self {
        Self {
            max_timestep,
            max_num_timestep_levels,
            current_lowest_allowed: TimestepLevel(max_num_timestep_levels - 1),
            first_update_at_highest_level_done: false,
        }
    }

    pub fn iter_levels_in_sweep_order(self) -> impl Iterator<Item = TimestepLevel> {
        let num = self.num_allowed_levels();
        (0..(2usize.pow(num as u32 - 1))).map(move |i| {
            self.current_lowest_allowed + self.lowest_active_from_iteration(i as u32, num as u32).0
        })
    }

    pub fn iter_allowed_levels(self) -> impl Iterator<Item = TimestepLevel> {
        (self.current_lowest_allowed.0..self.max_num_timestep_levels).map(TimestepLevel)
    }

    pub(crate) fn iter_all_levels(&self) -> impl Iterator<Item = TimestepLevel> {
        (0..self.max_num_timestep_levels).map(TimestepLevel)
    }

    pub fn advance_allowed_levels(&mut self) {
        // Decrease the lowest allowed timestep level but only if
        // we have already done one additional run with only the highest level.
        // Doing this aligns the timesteps with the max_timestep cadence since
        // Sum_{i=1}^n (Δt (1/2)^i) = Δt (1 - (1/2)^n)
        if self.first_update_at_highest_level_done && self.current_lowest_allowed.0 > 0 {
            self.current_lowest_allowed -= 1;
        }
        if !self.first_update_at_highest_level_done {
            self.first_update_at_highest_level_done = true;
        }
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

    fn lowest_active_from_iteration(&self, iteration: u32, num_levels: u32) -> TimestepLevel {
        // find the lowest set bit in iteration.
        assert!(self.max_num_timestep_levels < 32);
        let first_bit = find_index_of_lowest_set_bit_in_int(iteration).unwrap_or(num_levels - 1);
        TimestepLevel((num_levels - 1 - first_bit) as usize)
    }

    pub fn current_max_timestep(&self) -> Time {
        self.max_timestep * self.current_lowest_allowed.as_factor()
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

#[cfg(test)]
mod tests {
    use super::TimestepState;
    use crate::sweep::timestep_level::TimestepLevel;
    use crate::units::Time;

    #[test]
    fn lowest_allowed() {
        let mut state = TimestepState::new(Time::megayears(1.0), 5);
        assert_eq!(state.current_lowest_allowed, TimestepLevel(4));
        state.advance_allowed_levels();
        assert_eq!(state.current_lowest_allowed, TimestepLevel(4));
        state.advance_allowed_levels();
        assert_eq!(state.current_lowest_allowed, TimestepLevel(3));
        state.advance_allowed_levels();
        assert_eq!(state.current_lowest_allowed, TimestepLevel(2));
        state.advance_allowed_levels();
        assert_eq!(state.current_lowest_allowed, TimestepLevel(1));
        state.advance_allowed_levels();
        assert_eq!(state.current_lowest_allowed, TimestepLevel(0));
        state.advance_allowed_levels();
        assert_eq!(state.current_lowest_allowed, TimestepLevel(0));
    }

    #[test]
    fn iter_levels_in_sweep_order_advances_properly() {
        let mut state = TimestepState::new(Time::megayears(1.0), 5);
        let mut get_next_levels_iter = || {
            let levels = state
                .iter_levels_in_sweep_order()
                .map(|level| level.0)
                .collect::<Vec<_>>();
            state.advance_allowed_levels();
            levels
        };

        assert_eq!(get_next_levels_iter(), &[4]);
        assert_eq!(get_next_levels_iter(), &[4]);
        assert_eq!(get_next_levels_iter(), &[3, 4]);
        assert_eq!(get_next_levels_iter(), &[2, 4, 3, 4]);
        assert_eq!(get_next_levels_iter(), &[1, 4, 3, 4, 2, 4, 3, 4]);
        assert_eq!(
            get_next_levels_iter(),
            &[0, 4, 3, 4, 2, 4, 3, 4, 1, 4, 3, 4, 2, 4, 3, 4]
        );
    }
}
