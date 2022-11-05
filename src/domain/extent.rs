use mpi::traits::Equivalence;
use serde::Deserialize;
use serde::Deserializer;

use crate::config::TWO_TO_NUM_DIMENSIONS;
use crate::prelude::MVec;
use crate::units::Length;
use crate::units::VecLength;
use crate::units::Volume;

#[derive(Default, Clone, Equivalence, PartialEq)]
pub struct Extent {
    pub min: VecLength,
    pub max: VecLength,
    pub center: VecLength,
}

impl Extent {
    pub fn new(min: VecLength, max: VecLength) -> Self {
        debug_assert!(min.x() <= max.x());
        debug_assert!(min.y() <= max.y());
        #[cfg(not(feature = "2d"))]
        debug_assert!(min.z() <= max.z());
        Self {
            min,
            max,
            center: (min + max) * 0.5,
        }
    }

    pub fn cube_from_side_length(side_length: Length) -> Self {
        let min = VecLength::zero();
        let max = MVec::ONE * side_length;
        Self::new(min, max)
    }

    pub fn get_all_encompassing<'a>(extent: impl Iterator<Item = &'a Extent>) -> Option<Self> {
        Self::from_positions(
            extent.flat_map(|extent: &Extent| [&extent.min, &extent.max].into_iter()),
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

    pub fn center(&self) -> VecLength {
        self.center
    }

    pub fn side_lengths(&self) -> VecLength {
        self.max - self.min
    }

    pub fn max_side_length(&self) -> Length {
        let side_length = self.side_lengths();
        side_length.x().max(side_length.y())
    }

    pub fn from_positions<'a>(positions: impl Iterator<Item = &'a VecLength>) -> Option<Self> {
        let mut min = None;
        let mut max = None;
        let update_min = |min: &mut Option<VecLength>, pos: VecLength| {
            if let Some(ref mut min) = min {
                *min = min.min(pos);
            } else {
                *min = Some(pos);
            }
        };
        let update_max = |max: &mut Option<VecLength>, pos: VecLength| {
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
        Some(Self::new(min?, max?))
    }

    pub fn get_quadrant_index(&self, pos: &VecLength) -> usize {
        debug_assert!(self.contains(pos));
        #[cfg(feature = "2d")]
        match (pos.x() < self.center.x(), pos.y() < self.center.y()) {
            (true, true) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (false, false) => 3,
        }
        #[cfg(not(feature = "2d"))]
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

    pub fn contains(&self, pos: &VecLength) -> bool {
        self.min.x() <= pos.x()
            && pos.x() <= self.max.x()
            && self.min.y() <= pos.y()
            && pos.y() <= self.max.y()
    }

    pub fn volume(&self) -> Volume {
        let side_lengths = self.side_lengths();
        #[cfg(feature = "2d")]
        return side_lengths.x() * side_lengths.y();
        #[cfg(not(feature = "2d"))]
        return side_lengths.x() * side_lengths.y() * side_lengths.z();
    }

    #[cfg(feature = "2d")]
    pub fn get_quadrants(&self) -> [Self; TWO_TO_NUM_DIMENSIONS] {
        let min_00 = VecLength::new(self.min.x(), self.min.y());
        let min_10 = VecLength::new(self.center.x(), self.min.y());
        let min_01 = VecLength::new(self.min.x(), self.center.y());
        let min_11 = VecLength::new(self.center.x(), self.center.y());
        let max_00 = VecLength::new(self.center.x(), self.center.y());
        let max_10 = VecLength::new(self.max.x(), self.center.y());
        let max_01 = VecLength::new(self.center.x(), self.max.y());
        let max_11 = VecLength::new(self.max.x(), self.max.y());
        [
            Self::new(min_00, max_00),
            Self::new(min_10, max_10),
            Self::new(min_01, max_01),
            Self::new(min_11, max_11),
        ]
    }

    #[cfg(not(feature = "2d"))]
    pub fn get_quadrants(&self) -> [Self; TWO_TO_NUM_DIMENSIONS] {
        let min_000 = VecLength::new(self.min.x(), self.min.y(), self.min.z());
        let min_100 = VecLength::new(self.center.x(), self.min.y(), self.min.z());
        let min_010 = VecLength::new(self.min.x(), self.center.y(), self.min.z());
        let min_110 = VecLength::new(self.center.x(), self.center.y(), self.min.z());
        let min_001 = VecLength::new(self.min.x(), self.min.y(), self.center.z());
        let min_101 = VecLength::new(self.center.x(), self.min.y(), self.center.z());
        let min_011 = VecLength::new(self.min.x(), self.center.y(), self.center.z());
        let min_111 = VecLength::new(self.center.x(), self.center.y(), self.center.z());
        let max_000 = VecLength::new(self.center.x(), self.center.y(), self.center.z());
        let max_100 = VecLength::new(self.max.x(), self.center.y(), self.center.z());
        let max_010 = VecLength::new(self.center.x(), self.max.y(), self.center.z());
        let max_110 = VecLength::new(self.max.x(), self.max.y(), self.center.z());
        let max_001 = VecLength::new(self.center.x(), self.center.y(), self.max.z());
        let max_101 = VecLength::new(self.max.x(), self.center.y(), self.max.z());
        let max_011 = VecLength::new(self.center.x(), self.max.y(), self.max.z());
        let max_111 = VecLength::new(self.max.x(), self.max.y(), self.max.z());
        [
            Self::new(min_000, max_000),
            Self::new(min_100, max_100),
            Self::new(min_010, max_010),
            Self::new(min_110, max_110),
            Self::new(min_001, max_001),
            Self::new(min_101, max_101),
            Self::new(min_011, max_011),
            Self::new(min_111, max_111),
        ]
    }
}

/// A helper struct to enable deserialization of extents.
#[derive(Deserialize)]
#[serde(untagged)]
enum ExtentSpecification {
    MinMax { min: VecLength, max: VecLength },
    SideLength(Length),
}

impl From<ExtentSpecification> for Extent {
    fn from(value: ExtentSpecification) -> Self {
        match value {
            ExtentSpecification::MinMax { min, max } => Self::new(min, max),
            ExtentSpecification::SideLength(side_length) => {
                Extent::cube_from_side_length(side_length)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Extent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ExtentSpecification::deserialize(deserializer)?.into())
    }
}

impl std::fmt::Debug for Extent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Extent({:.3?} {:.3?})", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::MVec;
    use crate::units::VecLength;

    fn assert_is_close(a: VecLength, b: MVec) {
        assert!((a.in_meters() - b).length() < f64::EPSILON)
    }

    #[cfg(all(test, feature = "2d"))]
    mod two_d {
        use glam::DVec2;

        use super::assert_is_close;
        use crate::domain::Extent;
        use crate::prelude::MVec;
        use crate::units::VecLength;

        #[test]
        #[ignore]
        fn extent_quadrants() {
            let root_extent =
                Extent::new(VecLength::meters(-1.0, -2.0), VecLength::meters(1.0, 2.0));
            let quadrants = root_extent.get_quadrants();
            assert_is_close(quadrants[0].min, MVec::new(-1.0, -2.0));
            assert_is_close(quadrants[0].max, MVec::new(0.0, 0.0));

            assert_is_close(quadrants[1].min, MVec::new(0.0, -2.0));
            assert_is_close(quadrants[1].max, MVec::new(1.0, 0.0));

            assert_is_close(quadrants[2].min, MVec::new(-1.0, 0.0));
            assert_is_close(quadrants[2].max, MVec::new(0.0, 2.0));

            assert_is_close(quadrants[3].min, MVec::new(0.0, 0.0));
            assert_is_close(quadrants[3].max, MVec::new(1.0, 2.0));
        }

        #[test]
        #[ignore]
        fn extent_from_positions() {
            let positions = &[
                VecLength::meters(1.0, 0.0),
                VecLength::meters(-1.0, 0.0),
                VecLength::meters(0.0, -2.0),
                VecLength::meters(0.0, 2.0),
            ];
            let extent = Extent::from_positions(positions.iter()).unwrap();
            assert_is_close(extent.min, DVec2::new(-1.0, -2.0));
            assert_is_close(extent.max, DVec2::new(1.0, 2.0));
        }

        #[test]
        #[ignore]
        fn quadrant_index() {
            let root_extent =
                Extent::new(VecLength::meters(-1.0, -2.0), VecLength::meters(1.0, 2.0));
            for (i, quadrant) in root_extent.get_quadrants().iter().enumerate() {
                assert_eq!(i, root_extent.get_quadrant_index(&quadrant.center));
            }
        }
    }

    #[cfg(all(test, not(feature = "2d")))]
    mod three_d {
        use glam::DVec3;

        use super::super::Extent;
        use super::assert_is_close;
        use crate::prelude::MVec;
        use crate::units::Length;
        use crate::units::VecLength;

        #[test]
        fn extent_from_positions() {
            let positions = &[
                VecLength::meters(1.0, 0.0, -1.0),
                VecLength::meters(-1.0, 0.0, 0.0),
                VecLength::meters(0.0, -2.0, 0.0),
                VecLength::meters(0.0, 2.0, 1.0),
            ];
            let extent = Extent::from_positions(positions.iter()).unwrap();
            assert_is_close(extent.min, DVec3::new(-1.0, -2.0, -1.0));
            assert_is_close(extent.max, DVec3::new(1.0, 2.0, 1.0));
        }

        #[test]
        fn extent_quadrants() {
            let root_extent = Extent::new(
                VecLength::meters(-1.0, -2.0, -3.0),
                VecLength::meters(1.0, 2.0, 3.0),
            );
            let quadrants = root_extent.get_quadrants();
            assert_is_close(quadrants[0].min, MVec::new(-1.0, -2.0, -3.0));
            assert_is_close(quadrants[0].max, MVec::new(0.0, 0.0, 0.0));

            assert_is_close(quadrants[1].min, MVec::new(0.0, -2.0, -3.0));
            assert_is_close(quadrants[1].max, MVec::new(1.0, 0.0, 0.0));

            assert_is_close(quadrants[2].min, MVec::new(-1.0, 0.0, -3.0));
            assert_is_close(quadrants[2].max, MVec::new(0.0, 2.0, 0.0));

            assert_is_close(quadrants[3].min, MVec::new(0.0, 0.0, -3.0));
            assert_is_close(quadrants[3].max, MVec::new(1.0, 2.0, 0.0));

            assert_is_close(quadrants[4].min, MVec::new(-1.0, -2.0, 0.0));
            assert_is_close(quadrants[4].max, MVec::new(0.0, 0.0, 3.0));

            assert_is_close(quadrants[5].min, MVec::new(0.0, -2.0, 0.0));
            assert_is_close(quadrants[5].max, MVec::new(1.0, 0.0, 3.0));

            assert_is_close(quadrants[6].min, MVec::new(-1.0, 0.0, 0.0));
            assert_is_close(quadrants[6].max, MVec::new(0.0, 2.0, 3.0));

            assert_is_close(quadrants[7].min, MVec::new(0.0, 0.0, 0.0));
            assert_is_close(quadrants[7].max, MVec::new(1.0, 2.0, 3.0));
        }

        #[test]
        fn extent_from_positions_is_none_with_zero_positions() {
            assert!(Extent::from_positions([].iter()).is_none());
        }

        #[test]
        fn quadrant_index() {
            let root_extent = Extent::new(
                VecLength::meters(-1.0, -2.0, -3.0),
                VecLength::meters(1.0, 2.0, 3.0),
            );
            for (i, quadrant) in root_extent.get_quadrants().iter().enumerate() {
                assert_eq!(i, root_extent.get_quadrant_index(&quadrant.center));
            }
        }

        fn extent_equality(e1: &Extent, e2: &Extent) -> bool {
            (e1.min - e2.min).length() == Length::zero()
                && (e1.max - e2.max).length() == Length::zero()
        }

        #[test]
        fn deserialize() {
            let extent_from_side_length = serde_yaml::from_str::<Extent>("5 m").unwrap();
            assert!(extent_equality(
                &extent_from_side_length,
                &Extent::cube_from_side_length(Length::meters(5.0))
            ));
            let extent_from_min_max = serde_yaml::from_str::<Extent>(
                "
min: (0.0 0.0 0.0) m
max: (5.0 5.0 5.0) m",
            )
            .unwrap();
            assert!(extent_equality(
                &extent_from_min_max,
                &Extent::cube_from_side_length(Length::meters(5.0))
            ));
        }
    }
}
