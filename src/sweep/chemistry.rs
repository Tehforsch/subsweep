use std::fmt::Debug;
use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use bevy::prelude::Resource;
use mpi::traits::Equivalence;

use super::chemistry_solver::Solver;
use super::site::Site;
use crate::grid::Cell;
use crate::units::helpers::Float;
use crate::units::Density;
use crate::units::Dimensionless;
use crate::units::EnergyDensity;
use crate::units::Length;
use crate::units::PhotonFlux;
use crate::units::Temperature;
use crate::units::Time;
use crate::units::Volume;
use crate::units::PROTON_MASS;

pub trait Chemistry: Sized + 'static {
    type Photons: Photons;
    type Species: Debug;

    fn get_outgoing_flux(
        &self,
        cell: &Cell,
        site: &mut Site<Self>,
        incoming_flux: Self::Photons,
    ) -> Self::Photons;

    fn update_abundances(
        &self,
        site: &mut Site<Self>,
        flux: Self::Photons,
        timestep: Time,
        volume: Volume,
        length: Length,
    ) -> Time;
}

#[derive(Resource)]
pub struct HydrogenOnly {
    pub flux_treshold: PhotonFlux,
}

#[derive(Debug)]
pub struct HydrogenOnlySpecies {
    pub ionized_hydrogen_fraction: Dimensionless,
    pub internal_energy_density: EnergyDensity,
}

impl HydrogenOnlySpecies {
    pub(crate) fn new(
        ionized_hydrogen_fraction: Dimensionless,
        temperature: Temperature,
        density: Density,
    ) -> HydrogenOnlySpecies {
        let molecular_weight = 1.0 / (ionized_hydrogen_fraction + 1.0);
        let internal_energy = temperature.to_internal_energy(molecular_weight);
        let internal_energy_density = internal_energy * density;
        Self {
            ionized_hydrogen_fraction,
            internal_energy_density,
        }
    }
}

impl Site<HydrogenOnly> {
    pub(crate) fn get_temperature(&self, density: Density) -> Temperature {
        let molecular_weight = 1.0 / (self.species.ionized_hydrogen_fraction + 1.0);
        (self.species.internal_energy_density / density).to_temperature(molecular_weight)
    }
}

impl Chemistry for HydrogenOnly {
    type Photons = PhotonFlux;
    type Species = HydrogenOnlySpecies;

    fn get_outgoing_flux(
        &self,
        cell: &Cell,
        site: &mut Site<Self>,
        incoming_flux: Self::Photons,
    ) -> PhotonFlux {
        let neutral_hydrogen_number_density =
            site.density / PROTON_MASS * (1.0 - site.species.ionized_hydrogen_fraction);
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        if incoming_flux < self.flux_treshold {
            PhotonFlux::zero()
        } else {
            let absorbed_fraction = (-neutral_hydrogen_number_density * sigma * cell.size).exp();
            incoming_flux * absorbed_fraction
        }
    }

    fn update_abundances(
        &self,
        site: &mut Site<Self>,
        flux: Self::Photons,
        timestep: Time,
        volume: Volume,
        length: Length,
    ) -> Time {
        let old_fraction = site.species.ionized_hydrogen_fraction;
        let new_fraction = Solver {
            ionized_hydrogen_fraction: site.species.ionized_hydrogen_fraction,
            timestep,
            density: site.density,
            volume,
            length,
            flux,
        }
        .get_new_abundance();
        site.species.ionized_hydrogen_fraction = new_fraction;
        let relative_change = (old_fraction / (old_fraction - new_fraction)).abs();
        let change_timescale = relative_change * timestep;
        change_timescale
    }
}

pub trait Photons:
    Sum<Self>
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + Mul<Float, Output = Self>
    + Mul<Dimensionless, Output = Self>
    + Div<Float, Output = Self>
    + Debug
    + Clone
    + Equivalence
{
    fn zero() -> Self;
}

impl Photons for PhotonFlux {
    fn zero() -> Self {
        PhotonFlux::zero()
    }
}
