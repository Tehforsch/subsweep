pub mod hydrogen_only;
pub mod timescale;

use std::fmt::Debug;
use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use mpi::traits::Equivalence;

use self::timescale::Timescale;
use crate::sweep::grid::Cell;
use crate::sweep::site::Site;
use crate::units::helpers::Float;
use crate::units::Dimensionless;
use crate::units::Length;
use crate::units::PhotonRate;
use crate::units::Time;
use crate::units::Volume;

pub trait Chemistry: Sized + 'static {
    type Photons: Photons;
    type Species: Debug;

    fn get_outgoing_rate(
        &self,
        cell: &Cell,
        site: &Site<Self>,
        incoming_rate: Self::Photons,
    ) -> Self::Photons;

    fn update_abundances(
        &self,
        site: &mut Site<Self>,
        rate: Self::Photons,
        timestep: Time,
        volume: Volume,
        length: Length,
        trace: Option<u32>,
    ) -> Timescale;
}

pub trait Photons:
    Sum<Self>
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + Mul<Float, Output = Self>
    + Mul<Dimensionless, Output = Self>
    + Div<Float, Output = Self>
    + PartialOrd<Self>
    + Debug
    + Clone
    + Equivalence
{
    fn zero() -> Self;
    fn relative_change_to(&self, other: &Self) -> Dimensionless;
    fn below_threshold(&self, threshold: PhotonRate) -> bool;
    fn make_positive(&mut self) {
        if *self < Self::zero() {
            *self = Self::zero();
        }
    }
}

impl Photons for PhotonRate {
    fn zero() -> Self {
        PhotonRate::zero()
    }

    fn relative_change_to(&self, other: &Self) -> Dimensionless {
        ((*self - *other).abs() / *self)
            .abs()
            .min(1.0 / f64::EPSILON)
    }

    fn below_threshold(&self, threshold: PhotonRate) -> bool {
        self.abs() < threshold.abs()
    }
}
