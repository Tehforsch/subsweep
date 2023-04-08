use crate::units::Length;
use crate::units::MVec2;
use crate::units::MVec3;
use crate::units::Vec2Length;
use crate::units::Vec3Length;
use crate::voronoi::DVector;

pub trait Dimension {
    type Length;
    type Point: Clone + Copy + DVector + std::fmt::Debug;
    type UnitPoint: Clone + Copy + std::fmt::Debug;
}

#[derive(Clone, Debug)]
pub struct TwoD;
#[derive(Clone, Debug)]
pub struct ThreeD;

#[cfg(feature = "2d")]
pub type ActiveDimension = TwoD;
#[cfg(feature = "3d")]
pub type ActiveDimension = ThreeD;

impl Dimension for TwoD {
    type Length = Length;
    type Point = MVec2;
    type UnitPoint = Vec2Length;
}

impl Dimension for ThreeD {
    type Length = Length;
    type Point = MVec3;
    type UnitPoint = Vec3Length;
}
