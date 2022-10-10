#[cfg(feature = "2d")]
pub const NUM_DIMENSIONS: usize = 2;
#[cfg(not(feature = "2d"))]
pub const NUM_DIMENSIONS: usize = 3;

pub const TWO_TO_NUM_DIMENSIONS: usize = 2i32.pow(NUM_DIMENSIONS as u32) as usize;
