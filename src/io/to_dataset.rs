use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SystemDescriptor;
use hdf5::H5Type;

use super::output::create_dataset_system;
use super::output::plugin::IntoOutputSystem;
use super::output::timer::Timer;
use super::output::write_dataset_system;
use crate::units::Dimension;

#[derive(SystemLabel)]
struct DatasetSystemAmbiguityLabel;

pub trait ToDataset: Clone + H5Type + Sync + Send + 'static {
    fn dimension() -> Dimension;
    fn convert_base_units(self, factor: f64) -> Self;
    /// A static quantity does not change over the course of the
    /// simulation and only needs to be written to output once.
    fn is_static() -> bool {
        false
    }
}

impl<T: ToDataset + Component> IntoOutputSystem for T {
    fn write_system() -> SystemDescriptor {
        write_dataset_system::<T>
            .with_run_criteria(Timer::dataset_write_run_criterion::<T>)
            .into_descriptor()
            .label(DatasetSystemAmbiguityLabel)
            .ambiguous_with(DatasetSystemAmbiguityLabel)
    }

    fn create_system() -> SystemDescriptor {
        create_dataset_system::<T>
            .with_run_criteria(Timer::dataset_write_run_criterion::<T>)
            .into_descriptor()
            .label(DatasetSystemAmbiguityLabel)
            .ambiguous_with(DatasetSystemAmbiguityLabel)
    }
}
