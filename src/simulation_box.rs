use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use derive_custom::raxiom_parameters;
use derive_more::From;
use derive_more::Into;

use crate::domain::Extent;
use crate::prelude::Float;
use crate::units::Length;
use crate::units::VecLength;

/// The box size of the simulation. Periodic boundary conditions apply
/// beyond this box, meaning that the positions of particles outside
/// of this box are wrapped back into it.
#[raxiom_parameters("box_size")]
#[derive(From, Into, Deref, DerefMut, Debug)]
pub struct SimulationBox(Extent);

fn periodic_wrap_component(v: Float, min: Float, max: Float) -> Float {
    min + (v - min).rem_euclid(max - min)
}

fn minimize_component(v: Float, length: Float) -> Float {
    // The following only works if v is within [-3/2 L, 3/2 L].
    // where L = (box_end - box_start)
    // The debug assertion below will fail if the particle falls out of this range.
    // Since periodic_wrap always takes all particles to [box_start, box_end],
    // distances should always fulfill |v| <= 3/2 L = 3/2
    let v = if v >= length / 2.0 {
        v - length
    } else if v <= -length / 2.0 {
        v + length
    } else {
        v
    };
    debug_assert!(v.abs() <= length / 2.0);
    v
}

impl SimulationBox {
    pub fn cube_from_side_length(side_length: Length) -> Self {
        Self(Extent::cube_from_side_length(side_length))
    }

    pub fn cube_from_side_length_centered(side_length: Length) -> Self {
        Self(Extent::cube_from_side_length_centered(side_length))
    }

    pub fn periodic_wrap(&self, mut pos: VecLength) -> VecLength {
        pos.0.x = periodic_wrap_component(
            pos.0.x,
            self.min.x().value_unchecked(),
            self.max.x().value_unchecked(),
        );
        pos.0.y = periodic_wrap_component(
            pos.0.y,
            self.min.y().value_unchecked(),
            self.max.y().value_unchecked(),
        );
        #[cfg(not(feature = "2d"))]
        {
            pos.0.z = periodic_wrap_component(
                pos.0.z,
                self.min.z().value_unchecked(),
                self.max.z().value_unchecked(),
            );
        }
        pos
    }

    pub fn periodic_distance_vec(&self, p1: &VecLength, p2: &VecLength) -> VecLength {
        let mut dist = *p1 - *p2;
        let side_lengths = self.side_lengths();
        dist.0.x = minimize_component(
            dist.x().value_unchecked(),
            side_lengths.x().value_unchecked(),
        );
        dist.0.y = minimize_component(
            dist.y().value_unchecked(),
            side_lengths.y().value_unchecked(),
        );
        #[cfg(not(feature = "2d"))]
        {
            dist.0.z = minimize_component(
                dist.z().value_unchecked(),
                side_lengths.z().value_unchecked(),
            );
        }
        dist
    }

    pub fn periodic_distance(&self, p1: &VecLength, p2: &VecLength) -> Length {
        self.periodic_distance_vec(p1, p2).length()
    }
}

#[cfg(test)]
#[cfg(not(feature = "2d"))]
mod tests {
    use crate::domain::Extent;
    use crate::gravity::tests::get_particles;
    use crate::parameters::SimulationBox;
    use crate::test_utils::assert_is_close;
    use crate::test_utils::assert_vec_is_close;
    use crate::units::Length;
    use crate::units::VecLength;

    #[test]
    fn periodic_wrap() {
        let check_wrap = |box_: &SimulationBox, (x, y, z), (x_wrapped, y_wrapped, z_wrapped)| {
            let v = box_.periodic_wrap(VecLength::meters(x, y, z));
            assert_vec_is_close(v, VecLength::meters(x_wrapped, y_wrapped, z_wrapped));
        };
        let box_: SimulationBox = Extent::new(
            VecLength::meters(0.0, 0.0, 0.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_wrap(&box_, (0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (1.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (0.5, 2.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (0.5, 0.5, 3.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (1.5, 2.5, 3.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (-0.5, -0.5, -0.5), (0.5, 1.5, 2.5));
        let box_: SimulationBox = Extent::new(
            VecLength::meters(-1.0, -1.0, -1.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_wrap(&box_, (0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (-0.5, -0.5, -0.5), (-0.5, -0.5, -0.5));
        check_wrap(&box_, (-1.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_, (-1.5, -0.5, -0.5), (0.5, -0.5, -0.5));
    }

    #[test]
    fn periodic_distance() {
        let check_dist = |box_: &SimulationBox, (x1, y1, z1), (x2, y2, z2), distance| {
            let v1 = VecLength::meters(x1, y1, z1);
            let v2 = VecLength::meters(x2, y2, z2);
            assert_is_close(box_.periodic_distance(&v1, &v2), Length::meters(distance));
        };
        let box_: SimulationBox = Extent::new(
            VecLength::meters(0.0, 0.0, 0.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_dist(&box_, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.0);
        check_dist(&box_, (0.1, 0.0, 0.0), (0.1, 0.0, 0.0), 0.0);
        check_dist(&box_, (-0.1, 0.0, 0.0), (0.1, 0.0, 0.0), 0.2);
        check_dist(&box_, (0.0, -0.1, 0.0), (0.0, 0.1, 0.0), 0.2);
        check_dist(&box_, (0.0, 0.0, -0.1), (0.0, 0.0, 0.1), 0.2);
        check_dist(&box_, (0.0, 0.0, 0.0), (0.5, 0.0, 0.0), 0.5);
        check_dist(&box_, (0.2, 0.0, 0.0), (0.7, 0.0, 0.0), 0.5);
        let box_: SimulationBox = Extent::new(
            VecLength::meters(-1.0, -1.0, -1.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_dist(&box_, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.0);
        check_dist(&box_, (-1.1, 0.0, 0.0), (-0.9, 0.0, 0.0), 0.2);
        check_dist(&box_, (0.0, -1.1, 0.0), (0.0, -0.9, 0.0), 0.2);
        check_dist(&box_, (0.0, 0.0, -1.1), (0.0, 0.0, -0.9), 0.2);
        check_dist(&box_, (1.1, 0.0, 0.0), (0.9, 0.0, 0.0), 0.2);
        check_dist(&box_, (0.0, 2.1, 0.0), (0.0, 1.9, 0.0), 0.2);
        check_dist(&box_, (0.0, 0.0, 3.1), (0.0, 0.0, 2.9), 0.2);
    }

    #[test]
    fn periodic_distance_is_symmetric() {
        let particles = get_particles(5, 5);
        let box_: SimulationBox = Extent::new(
            VecLength::meters(-1.0, -1.0, -1.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        for p1 in particles.iter() {
            for p2 in particles.iter() {
                // Wrap the positions to make sure that we do not exceed the [-3/2 L, 3/2 L] interval in which periodic_distance_vec is valid
                let p1 = box_.periodic_wrap(p1.pos);
                let p2 = box_.periodic_wrap(p2.pos);
                let d1 = box_.periodic_distance_vec(&p1, &p2);
                let d2 = box_.periodic_distance_vec(&p2, &p1);
                assert_vec_is_close(d1, -d2);
            }
        }
    }
}
