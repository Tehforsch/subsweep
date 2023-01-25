#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
}
