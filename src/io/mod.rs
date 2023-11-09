pub mod file_distribution;
pub mod input;
pub mod output;
pub mod time_series;
pub mod to_dataset;
pub mod unit_reader;

use std::marker::PhantomData;

use bevy_ecs::prelude::Resource;
use hdf5::Dataset;
pub use unit_reader::DefaultUnitReader;
pub use unit_reader::UnitReader;

use crate::prelude::Float;
use crate::prelude::Named;
use crate::units::Dimension;

#[derive(Clone)]
pub struct DatasetDescriptor {
    pub dataset_name: String,
    pub unit_reader: Box<dyn UnitReader>,
}

impl DatasetDescriptor {
    pub fn default_for<T: Named>() -> Self {
        Self {
            dataset_name: T::name().into(),
            unit_reader: Box::new(DefaultUnitReader),
        }
    }

    pub fn dataset_name(&self) -> &str {
        &self.dataset_name
    }

    pub fn read_scale_factor(&self, set: &Dataset) -> f64 {
        self.unit_reader.read_scale_factor(set)
    }

    fn read_dimension(&self, set: &Dataset) -> Dimension {
        self.unit_reader.read_dimension(set)
    }
}

#[derive(Resource, Clone)]
pub enum DatasetShape<T> {
    OneDimensional,
    TwoDimensional(fn(&[Float]) -> T),
}

#[derive(Clone)]
pub struct InputDatasetDescriptor<T> {
    pub descriptor: DatasetDescriptor,
    pub shape: DatasetShape<T>,
}

impl<T: Named> Default for InputDatasetDescriptor<T> {
    fn default() -> Self {
        InputDatasetDescriptor {
            descriptor: DatasetDescriptor::default_for::<T>(),
            shape: DatasetShape::OneDimensional,
        }
    }
}

impl<T> InputDatasetDescriptor<T> {
    pub fn new(descriptor: DatasetDescriptor, shape: DatasetShape<T>) -> Self {
        Self { descriptor, shape }
    }
}

impl<T> std::ops::Deref for InputDatasetDescriptor<T> {
    type Target = DatasetDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.descriptor
    }
}

#[derive(Clone)]
pub struct OutputDatasetDescriptor<T> {
    _marker: PhantomData<T>,
    descriptor: DatasetDescriptor,
}

impl<T> OutputDatasetDescriptor<T> {
    fn new(descriptor: DatasetDescriptor) -> Self {
        Self {
            descriptor,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for OutputDatasetDescriptor<T> {
    type Target = DatasetDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.descriptor
    }
}
