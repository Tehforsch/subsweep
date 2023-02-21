use std::marker::PhantomData;

use bevy::prelude::Resource;

use crate::prelude::Float;
use crate::prelude::Named;

pub mod input;
pub mod output;
pub mod time_series;
pub mod to_dataset;

#[derive(Resource, Clone, Debug)]
pub struct DatasetDescriptor {
    pub dataset_name: String,
}

#[derive(Resource, Clone)]
pub enum DatasetShape<T> {
    OneDimensional,
    TwoDimensional(fn(&[Float]) -> T),
}

impl DatasetDescriptor {
    pub fn default_for<T: Named>() -> Self {
        Self {
            dataset_name: T::name().into(),
        }
    }

    pub fn dataset_name(&self) -> &str {
        &self.dataset_name
    }
}

#[derive(Resource, Clone)]
pub struct InputDatasetDescriptor<T> {
    _marker: PhantomData<T>,
    descriptor: DatasetDescriptor,
    pub shape: DatasetShape<T>,
}

impl<T> InputDatasetDescriptor<T> {
    pub fn new(descriptor: DatasetDescriptor, shape: DatasetShape<T>) -> Self {
        Self {
            descriptor,
            shape,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for InputDatasetDescriptor<T> {
    type Target = DatasetDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.descriptor
    }
}

#[derive(Resource, Clone)]
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
