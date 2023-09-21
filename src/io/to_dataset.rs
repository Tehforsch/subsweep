use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SystemDescriptor;
use hdf5::Dataset;
use hdf5::File;
use hdf5::H5Type;

use super::output::plugin::IntoOutputSystem;
use super::output::OutputFile;
use super::DatasetDescriptor;
use super::OutputDatasetDescriptor;
use crate::prelude::Particles;
use crate::units::Dimension;

pub const SCALE_FACTOR_IDENTIFIER: &str = "scale_factor_si";
pub const LENGTH_IDENTIFIER: &str = "scaling_length";
pub const TIME_IDENTIFIER: &str = "scaling_time";
pub const MASS_IDENTIFIER: &str = "scaling_mass";
pub const TEMPERATURE_IDENTIFIER: &str = "scaling_temperature";
pub const H_SCALING_IDENTIFIER: &str = "scaling_h";
pub const A_SCALING_IDENTIFIER: &str = "scaling_a";

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
    fn system() -> SystemDescriptor {
        write_dataset_system::<T>
            .into_descriptor()
            .label(DatasetSystemAmbiguityLabel)
            .ambiguous_with(DatasetSystemAmbiguityLabel)
    }
}

fn write_dataset_system<T: Component + ToDataset>(
    query: Particles<&T>,
    file: ResMut<OutputFile>,
    descriptor: NonSend<OutputDatasetDescriptor<T>>,
) {
    let data: Vec<T> = query.iter().cloned().collect();
    write_dataset(data, file.f.as_ref().unwrap(), &descriptor);
}

pub fn write_dataset<T: ToDataset>(data: Vec<T>, file: &File, descriptor: &DatasetDescriptor) {
    let dataset = file
        .new_dataset_builder()
        .with_data(&data)
        .create(descriptor.dataset_name())
        .expect("Failed to write dataset");
    add_dimension_attrs::<T>(&dataset);
}

pub fn add_dimension_attrs<T: ToDataset>(dataset: &Dataset) {
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
    let Dimension {
        length,
        time,
        mass,
        temperature,
        h,
        a,
    } = dimension;
    write_dimension(dataset, LENGTH_IDENTIFIER, length);
    write_dimension(dataset, TIME_IDENTIFIER, time);
    write_dimension(dataset, MASS_IDENTIFIER, mass);
    write_dimension(dataset, TEMPERATURE_IDENTIFIER, temperature);
    write_dimension(dataset, H_SCALING_IDENTIFIER, h);
    write_dimension(dataset, A_SCALING_IDENTIFIER, a);
}

fn write_dimension(dataset: &Dataset, identifier: &str, dimension: i32) {
    let attr = dataset
        .new_attr::<i32>()
        .shape(())
        .create(identifier)
        .unwrap();
    attr.write_scalar(&dimension).unwrap();
}
