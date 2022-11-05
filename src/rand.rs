use ::rand::Rng;

use crate::prelude::MVec;
use crate::units::Dimension;
use crate::units::Quantity;

#[cfg(feature = "2d")]
/// Generates random values in the range min..max.
/// ```
/// # use raxiom::units::VecLength;
/// # use raxiom::prelude::gen_range;
/// let min = VecLength::meters(0.0, 0.0);
/// let max = VecLength::meters(1.0, 5.0);
/// let length = gen_range(&mut rand::thread_rng(), min, max);
/// assert!(min.x() <= length.x() && length.x() < max.x());
/// assert!(min.y() <= length.y() && length.y() < max.y());
/// ```
pub fn gen_range<const D: Dimension, R: Rng>(
    rng: &mut R,
    min: Quantity<MVec, D>,
    max: Quantity<MVec, D>,
) -> Quantity<MVec, D> {
    Quantity::<MVec, D>::new(
        rng.gen_range(min.x()..max.x()),
        rng.gen_range(min.y()..max.y()),
    )
}

#[cfg(not(feature = "2d"))]
/// Generates random values in the range min..max.
/// ```
/// # use raxiom::units::VecLength;
/// # use raxiom::prelude::gen_range;
/// let min = VecLength::meters(0.0, 0.0, 0.0);
/// let max = VecLength::meters(1.0, 5.0, 3.0);
/// let length = gen_range(&mut rand::thread_rng(), min, max);
/// assert!(min.x() <= length.x() && length.x() < max.x());
/// assert!(min.y() <= length.y() && length.y() < max.y());
/// assert!(min.z() <= length.z() && length.z() < max.z());
/// ```
pub fn gen_range<const D: Dimension, R: Rng>(
    rng: &mut R,
    min: Quantity<MVec, D>,
    max: Quantity<MVec, D>,
) -> Quantity<MVec, D> {
    Quantity::<MVec, D>::new(
        rng.gen_range(min.x()..max.x()),
        rng.gen_range(min.y()..max.y()),
        rng.gen_range(min.z()..max.z()),
    )
}
