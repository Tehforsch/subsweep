#[cfg(feature = "2d")]
pub type Face = super::face_2d::Face2d;
#[cfg(feature = "3d")]
pub type Face = super::face_3d::Face3d;

#[cfg(feature = "3d")]
pub type FaceData = super::face_3d::Face3dData;
