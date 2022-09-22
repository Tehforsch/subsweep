use std::marker::PhantomData;

use bevy::prelude::Component;
use bevy::prelude::Query;
use bevy::prelude::ResMut;
use bevy::prelude::With;
use hdf5::Dataset;
use hdf5::H5Type;

use super::add_output_system;
use super::OutputFile;
use super::ShouldWriteOutput;
use crate::io::to_dataset::ToDataset;
use crate::named::Named;
use crate::physics::LocalParticle;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::Dimension;

pub const SCALE_FACTOR_IDENTIFIER: &str = "scale_factor_si";
pub const LENGTH_IDENTIFIER: &str = "scaling_length";
pub const TIME_IDENTIFIER: &str = "scaling_time";
pub const MASS_IDENTIFIER: &str = "scaling_mass";

#[derive(Named)]
pub struct DatasetOutputPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for DatasetOutputPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: ToDataset + Named + Clone + Component + Sync + Send + 'static> RaxiomPlugin
    for DatasetOutputPlugin<T>
{
    fn should_build(&self, sim: &Simulation) -> bool {
        sim.get_resource::<ShouldWriteOutput>()
            .map(|x| x.0)
            .unwrap_or(true)
    }

    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        add_output_system::<T, _>(sim, Self::write_dataset);
    }
}

impl<T: ToDataset + Named + Sync + Send + 'static + Clone + H5Type + Component>
    DatasetOutputPlugin<T>
{
    fn write_dataset(query: Query<&T, With<LocalParticle>>, file: ResMut<OutputFile>) {
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
}

fn write_dimension(dataset: &Dataset, identifier: &str, dimension: i32) {
    let attr = dataset
        .new_attr::<i32>()
        .shape(())
        .create(identifier)
        .unwrap();
    attr.write_scalar(&dimension).unwrap();
}
