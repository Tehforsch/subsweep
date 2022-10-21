use super::Dimension;
use super::Quantity;

/// The default floating point type.
pub type Float = f64;

#[cfg(feature = "2d")]
/// The default vector type.
pub type MVec = glam::DVec2;
#[cfg(not(feature = "2d"))]
/// The default vector type.
pub type MVec = glam::DVec3;

pub type VecQuantity<const D: Dimension> = Quantity<MVec, D>;

impl<const D: Dimension> Quantity<MVec, D> {
    /// Construct the vector from just x and y, filling z with zero,
    /// if available. This helps with constructing vectors in examples
    /// without having to differentiate between 2D and 3D
    pub fn from_xy(x: Quantity<f64, D>, y: Quantity<f64, D>) -> Self {
        #[cfg(feature = "2d")]
        return Self::new(x, y);
        #[cfg(not(feature = "2d"))]
        return Self::new(x, y, Quantity::<f64, D>::zero());
    }
}
