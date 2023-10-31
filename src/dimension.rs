use std::fmt::Debug;

use crate::domain::IntoKey;
use crate::extent::Extent;
use crate::prelude::Float;
use crate::simulation_box::PeriodicWrapType2d;
use crate::simulation_box::PeriodicWrapType3d;
use crate::units::Length;
use crate::units::MVec2;
use crate::units::MVec3;
use crate::units::Vec2Length;
use crate::units::Vec3Length;
use crate::voronoi::math::traits::DVector;
use crate::voronoi::visualizer::Visualizable;

pub trait Dimension {
    const NUM: i32;
    type Length;
    type Point: Clone + Copy + DVector<Float = Float> + IntoKey + Debug + Visualizable;
    type UnitPoint: Clone + Copy + IntoKey + Debug;
    type WrapType: Default + Clone + Debug + PartialEq + Eq + std::hash::Hash + Copy;
    fn remap_point(point: Point<Self>, extent: &Extent<Point<Self>>) -> Point<Self>;
}

pub type Point<D> = <D as Dimension>::Point;
pub type WrapType<D> = <D as Dimension>::WrapType;

#[derive(Clone, Debug, Default)]
pub struct TwoD;
#[derive(Clone, Debug, Default)]
pub struct ThreeD;

#[cfg(feature = "2d")]
pub type ActiveDimension = TwoD;
#[cfg(feature = "3d")]
pub type ActiveDimension = ThreeD;

impl Dimension for TwoD {
    const NUM: i32 = 2;
    type Length = Length;
    type Point = MVec2;
    type UnitPoint = Vec2Length;
    type WrapType = PeriodicWrapType2d;
    fn remap_point(point: Point<Self>, extent: &Extent<Point<Self>>) -> Point<Self> {
        debug_assert!(extent.contains(&point));
        MVec2::new(1.0, 1.0) + (point - extent.min) * (1.0 / extent.side_lengths())
    }
}

impl Dimension for ThreeD {
    const NUM: i32 = 3;
    type Length = Length;
    type Point = MVec3;
    type UnitPoint = Vec3Length;
    type WrapType = PeriodicWrapType3d;
    fn remap_point(point: Point<Self>, extent: &Extent<Point<Self>>) -> Point<Self> {
        debug_assert!(extent.contains(&point));
        MVec3::new(1.0, 1.0, 1.0) + (point - extent.min) * (1.0 / extent.side_lengths())
    }
}

#[cfg(feature = "2d")]
pub type ActiveWrapType = crate::simulation_box::PeriodicWrapType2d;
#[cfg(feature = "3d")]
pub type ActiveWrapType = crate::simulation_box::PeriodicWrapType3d;
