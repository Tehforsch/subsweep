use serde::Deserialize;
use serde::Deserializer;

use crate::domain::key::IntoKey;
use crate::extent::Extent as VExtent;
use crate::peano_hilbert::PeanoKey2d;
use crate::peano_hilbert::PeanoKey3d;
use crate::units::Area;
use crate::units::Length;
use crate::units::MVec2;
use crate::units::MVec3;
use crate::units::Vec2Length;
use crate::units::Vec3Length;
use crate::units::Volume;

pub type Extent2d = VExtent<Vec2Length>;
pub type Extent3d = VExtent<Vec3Length>;

#[cfg(feature = "2d")]
pub type Extent = Extent2d;
#[cfg(feature = "3d")]
pub type Extent = Extent3d;

macro_rules! impl_extent {
    ($extent: ident, $spec: ident, $unit_vec: ident, $vec: ident, $length: ident, $key: ident) => {
        impl $extent {
            pub fn cube_from_side_length(side_length: $length) -> Self {
                let min = $unit_vec::zero();
                let max = $vec::ONE * side_length;
                Self::from_min_max(min, max)
            }

            pub fn cube_from_side_length_centered(side_length: $length) -> Self {
                let min = -$vec::ONE * side_length / 2.0;
                let max = $vec::ONE * side_length / 2.0;
                Self::from_min_max(min, max)
            }

            pub fn get_all_encompassing<'a>(
                extent: impl Iterator<Item = &'a Self>,
            ) -> Option<Self> {
                Self::from_positions(
                    extent.flat_map(|extent: &Self| [&extent.min, &extent.max].into_iter()),
                )
            }

            /// Return an extent with slightly increased size
            /// but the same center
            pub fn pad(self) -> Self {
                let dist_to_min = self.min - self.center;
                let dist_to_max = self.max - self.center;
                const PADDING_FRACTION: f64 = 0.01;
                Self {
                    min: self.center + dist_to_min * (1.0 + PADDING_FRACTION),
                    max: self.center + dist_to_max * (1.0 + PADDING_FRACTION),
                    center: self.center,
                }
            }

            pub fn center(&self) -> $unit_vec {
                self.center
            }

            pub fn side_lengths(&self) -> $unit_vec {
                self.max - self.min
            }

            pub fn from_positions<'a>(
                positions: impl Iterator<Item = &'a $unit_vec>,
            ) -> Option<Self> {
                let mut min = None;
                let mut max = None;
                let update_min = |min: &mut Option<$unit_vec>, pos: $unit_vec| {
                    if let Some(ref mut min) = min {
                        *min = min.min(pos);
                    } else {
                        *min = Some(pos);
                    }
                };
                let update_max = |max: &mut Option<$unit_vec>, pos: $unit_vec| {
                    if let Some(ref mut max) = max {
                        *max = max.max(pos);
                    } else {
                        *max = Some(pos);
                    }
                };
                for pos in positions {
                    update_min(&mut min, *pos);
                    update_max(&mut max, *pos);
                }
                Some(Self::from_min_max(min?, max?))
            }

            pub fn contains_extent(&self, other: &Self) -> bool {
                self.contains(&other.min) && self.contains(&other.max)
            }
        }

        /// A helper struct to enable deserialization of extents.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum $spec {
            MinMax { min: $unit_vec, max: $unit_vec },
            Size($unit_vec),
            SideLength($length),
        }

        impl From<$spec> for $extent {
            fn from(value: $spec) -> Self {
                match value {
                    $spec::MinMax { min, max } => Self::from_min_max(min, max),
                    $spec::Size(size) => Self::from_min_max($unit_vec::zero(), size),
                    $spec::SideLength(side_length) => $extent::cube_from_side_length(side_length),
                }
            }
        }

        impl<'de> Deserialize<'de> for $extent {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok($spec::deserialize(deserializer)?.into())
            }
        }

        impl std::fmt::Debug for $extent {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Extent({:.3?} {:.3?})", self.min, self.max)
            }
        }
    };
}

impl_extent!(
    Extent2d,
    ExtentSpecification2d,
    Vec2Length,
    MVec2,
    Length,
    PeanoKey2d
);
impl_extent!(
    Extent3d,
    ExtentSpecification3d,
    Vec3Length,
    MVec3,
    Length,
    PeanoKey3d
);

impl VExtent<MVec2> {
    pub fn cube_around_sphere(center: MVec2, radius: f64) -> Self {
        let min = center - MVec2::ONE * radius;
        let max = center + MVec2::ONE * radius;
        Self { center, min, max }
    }

    pub fn get_min_and_max_key(&self, global_extent: &Self) -> (PeanoKey2d, PeanoKey2d) {
        let keys: Vec<_> = self
            .get_extreme_points()
            .iter()
            .map(|p| p.into_key(global_extent))
            .collect();
        (*keys.iter().min().unwrap(), *keys.iter().max().unwrap())
    }

    pub fn get_extreme_points(&self) -> [MVec2; 4] {
        [
            MVec2::new(self.min.x, self.min.y),
            MVec2::new(self.max.x, self.min.y),
            MVec2::new(self.max.x, self.max.y),
            MVec2::new(self.min.x, self.max.y),
        ]
    }
}

impl VExtent<MVec3> {
    pub fn cube_around_sphere(center: MVec3, radius: f64) -> Self {
        let min = center - MVec3::ONE * radius;
        let max = center + MVec3::ONE * radius;
        Self { center, min, max }
    }

    pub fn get_min_and_max_key(&self, global_extent: &Self) -> (PeanoKey3d, PeanoKey3d) {
        let keys: Vec<_> = self
            .get_extreme_points()
            .iter()
            .map(|p| p.into_key(global_extent))
            .collect();
        (*keys.iter().min().unwrap(), *keys.iter().max().unwrap())
    }

    pub fn get_extreme_points(&self) -> [MVec3; 8] {
        [
            MVec3::new(self.min.x, self.min.y, self.min.z),
            MVec3::new(self.max.x, self.min.y, self.min.z),
            MVec3::new(self.max.x, self.max.y, self.min.z),
            MVec3::new(self.min.x, self.max.y, self.min.z),
            MVec3::new(self.min.x, self.min.y, self.max.z),
            MVec3::new(self.max.x, self.min.y, self.max.z),
            MVec3::new(self.max.x, self.max.y, self.max.z),
            MVec3::new(self.min.x, self.max.y, self.max.z),
        ]
    }
}

impl Extent2d {
    pub fn get_quadrant_index(&self, pos: &Vec2Length) -> usize {
        debug_assert!(self.contains(pos));
        match (pos.x() < self.center.x(), pos.y() < self.center.y()) {
            (true, true) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (false, false) => 3,
        }
    }
    pub fn get_quadrants(&self) -> [Self; 4] {
        let min_00 = Vec2Length::new(self.min.x(), self.min.y());
        let min_10 = Vec2Length::new(self.center.x(), self.min.y());
        let min_01 = Vec2Length::new(self.min.x(), self.center.y());
        let min_11 = Vec2Length::new(self.center.x(), self.center.y());
        let max_00 = Vec2Length::new(self.center.x(), self.center.y());
        let max_10 = Vec2Length::new(self.max.x(), self.center.y());
        let max_01 = Vec2Length::new(self.center.x(), self.max.y());
        let max_11 = Vec2Length::new(self.max.x(), self.max.y());
        [
            Self::from_min_max(min_00, max_00),
            Self::from_min_max(min_10, max_10),
            Self::from_min_max(min_01, max_01),
            Self::from_min_max(min_11, max_11),
        ]
    }

    pub fn max_side_length(&self) -> Length {
        let side_length = self.side_lengths();
        side_length.x().max(side_length.y())
    }

    pub fn contains(&self, pos: &Vec2Length) -> bool {
        self.min.x() <= pos.x()
            && pos.x() <= self.max.x()
            && self.min.y() <= pos.y()
            && pos.y() <= self.max.y()
    }

    pub fn volume(&self) -> Area {
        let s = self.side_lengths();
        s.x() * s.y()
    }
}

impl Extent3d {
    pub fn get_quadrant_index(&self, pos: &Vec3Length) -> usize {
        match (
            pos.x() < self.center.x(),
            pos.y() < self.center.y(),
            pos.z() < self.center.z(),
        ) {
            (true, true, true) => 0,
            (false, true, true) => 1,
            (true, false, true) => 2,
            (false, false, true) => 3,
            (true, true, false) => 4,
            (false, true, false) => 5,
            (true, false, false) => 6,
            (false, false, false) => 7,
        }
    }
    pub fn get_quadrants(&self) -> [Self; 8] {
        let min_000 = Vec3Length::new(self.min.x(), self.min.y(), self.min.z());
        let min_100 = Vec3Length::new(self.center.x(), self.min.y(), self.min.z());
        let min_010 = Vec3Length::new(self.min.x(), self.center.y(), self.min.z());
        let min_110 = Vec3Length::new(self.center.x(), self.center.y(), self.min.z());
        let min_001 = Vec3Length::new(self.min.x(), self.min.y(), self.center.z());
        let min_101 = Vec3Length::new(self.center.x(), self.min.y(), self.center.z());
        let min_011 = Vec3Length::new(self.min.x(), self.center.y(), self.center.z());
        let min_111 = Vec3Length::new(self.center.x(), self.center.y(), self.center.z());
        let max_000 = Vec3Length::new(self.center.x(), self.center.y(), self.center.z());
        let max_100 = Vec3Length::new(self.max.x(), self.center.y(), self.center.z());
        let max_010 = Vec3Length::new(self.center.x(), self.max.y(), self.center.z());
        let max_110 = Vec3Length::new(self.max.x(), self.max.y(), self.center.z());
        let max_001 = Vec3Length::new(self.center.x(), self.center.y(), self.max.z());
        let max_101 = Vec3Length::new(self.max.x(), self.center.y(), self.max.z());
        let max_011 = Vec3Length::new(self.center.x(), self.max.y(), self.max.z());
        let max_111 = Vec3Length::new(self.max.x(), self.max.y(), self.max.z());
        [
            Self::from_min_max(min_000, max_000),
            Self::from_min_max(min_100, max_100),
            Self::from_min_max(min_010, max_010),
            Self::from_min_max(min_110, max_110),
            Self::from_min_max(min_001, max_001),
            Self::from_min_max(min_101, max_101),
            Self::from_min_max(min_011, max_011),
            Self::from_min_max(min_111, max_111),
        ]
    }

    pub fn max_side_length(&self) -> Length {
        let side_length = self.side_lengths();
        side_length.x().max(side_length.y()).max(side_length.z())
    }

    pub fn contains(&self, pos: &Vec3Length) -> bool {
        self.min.x() <= pos.x()
            && pos.x() <= self.max.x()
            && self.min.y() <= pos.y()
            && pos.y() <= self.max.y()
            && self.min.z() <= pos.z()
            && pos.z() <= self.max.z()
    }

    pub fn volume(&self) -> Volume {
        let s = self.side_lengths();
        s.x() * s.y() * s.z()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::extent::Extent3d;

    fn assert_is_close_2d(a: Vec2Length, b: MVec2) {
        assert!((a.in_meters() - b).length() < f64::EPSILON)
    }

    fn assert_is_close_3d(a: Vec3Length, b: MVec3) {
        assert!((a.in_meters() - b).length() < f64::EPSILON)
    }

    use super::Extent2d;
    use crate::units::Length;
    use crate::units::MVec2;
    use crate::units::MVec3;
    use crate::units::Vec2Length;
    use crate::units::Vec3Length;

    #[test]
    #[ignore]
    fn extent_quadrants_2d() {
        let root_extent =
            Extent2d::from_min_max(Vec2Length::meters(-1.0, -2.0), Vec2Length::meters(1.0, 2.0));
        let quadrants = root_extent.get_quadrants();
        assert_is_close_2d(quadrants[0].min, MVec2::new(-1.0, -2.0));
        assert_is_close_2d(quadrants[0].max, MVec2::new(0.0, 0.0));

        assert_is_close_2d(quadrants[1].min, MVec2::new(0.0, -2.0));
        assert_is_close_2d(quadrants[1].max, MVec2::new(1.0, 0.0));

        assert_is_close_2d(quadrants[2].min, MVec2::new(-1.0, 0.0));
        assert_is_close_2d(quadrants[2].max, MVec2::new(0.0, 2.0));

        assert_is_close_2d(quadrants[3].min, MVec2::new(0.0, 0.0));
        assert_is_close_2d(quadrants[3].max, MVec2::new(1.0, 2.0));
    }

    #[test]
    #[ignore]
    fn extent_from_positions_2d() {
        let positions = &[
            Vec2Length::meters(1.0, 0.0),
            Vec2Length::meters(-1.0, 0.0),
            Vec2Length::meters(0.0, -2.0),
            Vec2Length::meters(0.0, 2.0),
        ];
        let extent = Extent2d::from_positions(positions.iter()).unwrap();
        assert_is_close_2d(extent.min, MVec2::new(-1.0, -2.0));
        assert_is_close_2d(extent.max, MVec2::new(1.0, 2.0));
    }

    #[test]
    #[ignore]
    fn quadrant_index_2d() {
        let root_extent =
            Extent2d::from_min_max(Vec2Length::meters(-1.0, -2.0), Vec2Length::meters(1.0, 2.0));
        for (i, quadrant) in root_extent.get_quadrants().iter().enumerate() {
            assert_eq!(i, root_extent.get_quadrant_index(&quadrant.center));
        }
    }

    #[test]
    fn extent_from_positions_3d() {
        let positions = &[
            Vec3Length::meters(1.0, 0.0, -1.0),
            Vec3Length::meters(-1.0, 0.0, 0.0),
            Vec3Length::meters(0.0, -2.0, 0.0),
            Vec3Length::meters(0.0, 2.0, 1.0),
        ];
        let extent = Extent3d::from_positions(positions.iter()).unwrap();
        assert_is_close_3d(extent.min, MVec3::new(-1.0, -2.0, -1.0));
        assert_is_close_3d(extent.max, MVec3::new(1.0, 2.0, 1.0));
    }

    #[test]
    fn extent_quadrants_3d() {
        let root_extent = Extent3d::from_min_max(
            Vec3Length::meters(-1.0, -2.0, -3.0),
            Vec3Length::meters(1.0, 2.0, 3.0),
        );
        let quadrants = root_extent.get_quadrants();
        assert_is_close_3d(quadrants[0].min, MVec3::new(-1.0, -2.0, -3.0));
        assert_is_close_3d(quadrants[0].max, MVec3::new(0.0, 0.0, 0.0));

        assert_is_close_3d(quadrants[1].min, MVec3::new(0.0, -2.0, -3.0));
        assert_is_close_3d(quadrants[1].max, MVec3::new(1.0, 0.0, 0.0));

        assert_is_close_3d(quadrants[2].min, MVec3::new(-1.0, 0.0, -3.0));
        assert_is_close_3d(quadrants[2].max, MVec3::new(0.0, 2.0, 0.0));

        assert_is_close_3d(quadrants[3].min, MVec3::new(0.0, 0.0, -3.0));
        assert_is_close_3d(quadrants[3].max, MVec3::new(1.0, 2.0, 0.0));

        assert_is_close_3d(quadrants[4].min, MVec3::new(-1.0, -2.0, 0.0));
        assert_is_close_3d(quadrants[4].max, MVec3::new(0.0, 0.0, 3.0));

        assert_is_close_3d(quadrants[5].min, MVec3::new(0.0, -2.0, 0.0));
        assert_is_close_3d(quadrants[5].max, MVec3::new(1.0, 0.0, 3.0));

        assert_is_close_3d(quadrants[6].min, MVec3::new(-1.0, 0.0, 0.0));
        assert_is_close_3d(quadrants[6].max, MVec3::new(0.0, 2.0, 3.0));

        assert_is_close_3d(quadrants[7].min, MVec3::new(0.0, 0.0, 0.0));
        assert_is_close_3d(quadrants[7].max, MVec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn extent_from_positions_is_none_with_zero_positions() {
        assert!(Extent3d::from_positions([].iter()).is_none());
    }

    #[test]
    fn quadrant_index() {
        let root_extent = Extent3d::from_min_max(
            Vec3Length::meters(-1.0, -2.0, -3.0),
            Vec3Length::meters(1.0, 2.0, 3.0),
        );
        for (i, quadrant) in root_extent.get_quadrants().iter().enumerate() {
            assert_eq!(i, root_extent.get_quadrant_index(&quadrant.center));
        }
    }

    fn extent_equality(e1: &Extent3d, e2: &Extent3d) -> bool {
        (e1.min - e2.min).length() == Length::zero() && (e1.max - e2.max).length() == Length::zero()
    }

    #[test]
    fn deserialize() {
        let extent_from_side_length = serde_yaml::from_str::<Extent3d>("5 m").unwrap();
        assert!(extent_equality(
            &extent_from_side_length,
            &Extent3d::cube_from_side_length(Length::meters(5.0))
        ));
        let extent_from_min_max = serde_yaml::from_str::<Extent3d>(
            "
min: (0.0 0.0 0.0) m
max: (5.0 5.0 5.0) m",
        )
        .unwrap();
        assert!(extent_equality(
            &extent_from_min_max,
            &Extent3d::cube_from_side_length(Length::meters(5.0))
        ));
    }
}
