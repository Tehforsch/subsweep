use super::Dimension;
use super::Dimensionless;
use super::EnergyPerMass;
use super::Length;
use super::Quantity;
use super::Temperature;
use super::VecLength;
use super::BOLTZMANN_CONSTANT;
use super::GAMMA;
use super::PROTON_MASS;
use crate::parameters::BoxSize;
use crate::prelude::Float;

impl<const D: Dimension> Quantity<Float, D> {
    pub fn one_unchecked() -> Self {
        Self(1.0)
    }
}

fn periodic_wrap_component(v: Float, min: Float, max: Float) -> Float {
    (v - min).rem_euclid(max - min) + min
}

fn minimize_component(v: Float, length: Float) -> Float {
    if v < 0.0 {
        if v.abs() < (v + length).abs() {
            v
        } else {
            v + length
        }
    } else if v.abs() < (v - length).abs() {
        v
    } else {
        length - v
    }
}

impl VecLength {
    pub fn periodic_wrap(mut self, box_size: &BoxSize) -> Self {
        self.0.x = periodic_wrap_component(
            self.0.x,
            box_size.min.x().value_unchecked(),
            box_size.max.x().value_unchecked(),
        );
        self.0.y = periodic_wrap_component(
            self.0.y,
            box_size.min.y().value_unchecked(),
            box_size.max.y().value_unchecked(),
        );
        #[cfg(not(feature = "2d"))]
        {
            self.0.z = periodic_wrap_component(
                self.0.z,
                box_size.min.z().value_unchecked(),
                box_size.max.z().value_unchecked(),
            );
        }
        self
    }

    pub fn periodic_distance_vec(&self, other: &VecLength, box_size: &BoxSize) -> VecLength {
        let mut dist = *self - *other;
        let side_lengths = box_size.side_lengths();
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

    pub fn periodic_distance(&self, other: &VecLength, box_size: &BoxSize) -> Length {
        self.periodic_distance_vec(other, box_size).length()
    }
}

impl Temperature {
    pub fn to_internal_energy(&self, molecular_weight: Dimensionless) -> EnergyPerMass {
        *self * (BOLTZMANN_CONSTANT / PROTON_MASS) * (1.0 / (GAMMA - 1.0)) / molecular_weight
    }
}

impl EnergyPerMass {
    pub fn to_temperature(&self, molecular_weight: Dimensionless) -> Temperature {
        *self / (BOLTZMANN_CONSTANT / PROTON_MASS) / (1.0 / (GAMMA - 1.0)) * molecular_weight
    }
}

#[cfg(test)]
#[cfg(not(feature = "2d"))]
mod tests {
    use crate::domain::Extent;
    use crate::parameters::BoxSize;
    use crate::test_utils::assert_is_close;
    use crate::test_utils::assert_vec_is_close;
    use crate::units::Length;
    use crate::units::VecLength;

    #[test]
    fn periodic_wrap() {
        let check_wrap = |box_size, (x, y, z), (x_wrapped, y_wrapped, z_wrapped)| {
            let v = VecLength::meters(x, y, z).periodic_wrap(box_size);
            assert_vec_is_close(v, VecLength::meters(x_wrapped, y_wrapped, z_wrapped));
        };
        let box_size: BoxSize = Extent::new(
            VecLength::meters(0.0, 0.0, 0.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_wrap(&box_size, (0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (1.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (0.5, 2.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (0.5, 0.5, 3.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (1.5, 2.5, 3.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (-0.5, -0.5, -0.5), (0.5, 1.5, 2.5));
        let box_size: BoxSize = Extent::new(
            VecLength::meters(-1.0, -1.0, -1.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_wrap(&box_size, (0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (-0.5, -0.5, -0.5), (-0.5, -0.5, -0.5));
        check_wrap(&box_size, (-1.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        check_wrap(&box_size, (-1.5, -0.5, -0.5), (0.5, -0.5, -0.5));
    }

    #[test]
    fn periodic_distance() {
        let check_dist = |box_size, (x1, y1, z1), (x2, y2, z2), distance| {
            let v1 = VecLength::meters(x1, y1, z1);
            let v2 = VecLength::meters(x2, y2, z2);
            assert_is_close(
                v1.periodic_distance(&v2, box_size),
                Length::meters(distance),
            );
        };
        let box_size: BoxSize = Extent::new(
            VecLength::meters(0.0, 0.0, 0.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_dist(&box_size, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.0);
        check_dist(&box_size, (0.1, 0.0, 0.0), (0.1, 0.0, 0.0), 0.0);
        check_dist(&box_size, (-0.1, 0.0, 0.0), (0.1, 0.0, 0.0), 0.2);
        check_dist(&box_size, (0.0, -0.1, 0.0), (0.0, 0.1, 0.0), 0.2);
        check_dist(&box_size, (0.0, 0.0, -0.1), (0.0, 0.0, 0.1), 0.2);
        check_dist(&box_size, (0.0, 0.0, 0.0), (0.5, 0.0, 0.0), 0.5);
        check_dist(&box_size, (0.2, 0.0, 0.0), (0.7, 0.0, 0.0), 0.5);
        let box_size: BoxSize = Extent::new(
            VecLength::meters(-1.0, -1.0, -1.0),
            VecLength::meters(1.0, 2.0, 3.0),
        )
        .into();
        check_dist(&box_size, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 0.0);
        check_dist(&box_size, (-1.1, 0.0, 0.0), (-0.9, 0.0, 0.0), 0.2);
        check_dist(&box_size, (0.0, -1.1, 0.0), (0.0, -0.9, 0.0), 0.2);
        check_dist(&box_size, (0.0, 0.0, -1.1), (0.0, 0.0, -0.9), 0.2);
        check_dist(&box_size, (1.1, 0.0, 0.0), (0.9, 0.0, 0.0), 0.2);
        check_dist(&box_size, (0.0, 2.1, 0.0), (0.0, 1.9, 0.0), 0.2);
        check_dist(&box_size, (0.0, 0.0, 3.1), (0.0, 0.0, 2.9), 0.2);
    }
}
