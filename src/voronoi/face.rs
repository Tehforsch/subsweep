use super::primitives::line::Line;
use super::primitives::triangle::Triangle;
use super::primitives::triangle::TriangleData;
use super::primitives::Point3d;

#[cfg(feature = "2d")]
pub type Face = Line;
#[cfg(feature = "3d")]
pub type Face = Triangle;

#[cfg(feature = "3d")]
pub type FaceData = TriangleData<Point3d>;
