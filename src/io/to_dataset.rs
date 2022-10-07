use std::ops::Deref;

use bevy::ecs::schedule::ParallelSystemDescriptor;
use bevy::prelude::*;
use hdf5::Dataset;
use hdf5::H5Type;

use super::output::plugin::IntoOutputSystem;
use super::output::OutputFile;
use super::output::OutputSystemsAmbiguitySet;
use crate::named::Named;
use crate::prelude::LocalParticle;
use crate::units::Dimension;
use crate::units::Quantity;

pub const SCALE_FACTOR_IDENTIFIER: &str = "scale_factor_si";
pub const LENGTH_IDENTIFIER: &str = "scaling_length";
pub const TIME_IDENTIFIER: &str = "scaling_time";
pub const MASS_IDENTIFIER: &str = "scaling_mass";

pub trait ToDataset: Clone + Component + H5Type + Named + Sync + Send + 'static {
    fn dimension() -> Dimension;
    fn convert_base_units(self, factor: f64) -> Self;
}

impl<const D: Dimension, S, T> ToDataset for T
where
    S: Clone + 'static + std::ops::Mul<f64, Output = S>,
    T: Clone + Component + Named + H5Type + Deref<Target = Quantity<S, D>> + From<Quantity<S, D>>,
{
    fn dimension() -> Dimension {
        D
    }

    fn convert_base_units(self, factor: f64) -> T {
        (T::deref(&self).clone() * factor).into()
    }
}

impl<T: ToDataset> IntoOutputSystem for T {
    fn system() -> ParallelSystemDescriptor {
        write_dataset::<T>.in_ambiguity_set(OutputSystemsAmbiguitySet)
    }
}

fn write_dataset<T: ToDataset>(query: Query<&T, With<LocalParticle>>, file: ResMut<OutputFile>) {
    let f = file.f.as_ref().unwrap();
    let data: Vec<T> = query.iter().cloned().collect();
    let dataset = f
        .new_dataset_builder()
        .with_data(&data)
        .create(T::name())
        .expect("Failed to write dataset");
    let attr = dataset
        .new_attr::<f64>()
        .shape(())
        .create(SCALE_FACTOR_IDENTIFIER)
        .unwrap();
    let dimension = T::dimension();
    let scale_factor = dimension.base_conversion_factor();
    attr.write_scalar(&scale_factor).unwrap();
    // Unpack this slightly awkwardly here to make sure that we
    // remember to extend it once more units are added to the
    // Dimension struct
    let Dimension { length, time, mass } = dimension;
    write_dimension(&dataset, LENGTH_IDENTIFIER, length);
    write_dimension(&dataset, TIME_IDENTIFIER, time);
    write_dimension(&dataset, MASS_IDENTIFIER, mass);
}

fn write_dimension(dataset: &Dataset, identifier: &str, dimension: i32) {
    let attr = dataset
        .new_attr::<i32>()
        .shape(())
        .create(identifier)
        .unwrap();
    attr.write_scalar(&dimension).unwrap();
}
