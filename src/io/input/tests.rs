use std::path::Path;

use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::Query;
use bevy_ecs::prelude::World;

use super::read_dataset_system;
use super::InputParameters;
use super::SpawnedEntities;
use crate::components::Mass;
use crate::io::to_dataset::ToDataset;
use crate::io::DatasetDescriptor;
use crate::io::DatasetShape;
use crate::io::InputDatasetDescriptor;
use crate::prelude::Named;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::test_utils::assert_is_close;
use crate::test_utils::run_system_on_world;
use crate::test_utils::tests_path;
use crate::units::{self};

#[test]
fn respect_scale_factor() {
    let mut world = World::new();
    read_dataset_from_file::<Mass>(
        &mut world,
        &tests_path().join("input/respect_scale_factor.hdf5"),
    );
    run_system_on_world(&mut world, check_value_system);
}

fn check_value_system(query: Query<&Mass>) {
    // The file contains a single particle with the "numerical value"
    // of 1 solar mass, but a SI scale factor of 5 (because it was
    // written with a different base unit system). Make sure the value
    // is converted properly
    let mass = **query.single();
    assert_is_close(mass, units::Mass::solar(5.0));
}

#[test]
#[should_panic(expected = "Mismatch in dimension while reading dataset mass.")]
fn panic_on_dimension_mismatch() {
    let mut world = World::new();
    read_dataset_from_file::<Mass>(
        &mut world,
        &tests_path().join("input/panic_on_dimension_mismatch.hdf5"),
    );
    run_system_on_world(&mut world, check_value_system);
}

fn read_dataset_from_file<T: ToDataset + Component + Named>(world: &mut World, file: &Path) {
    let entity = world.spawn_empty().id();
    world.insert_resource(SpawnedEntities(vec![entity]));
    world.insert_resource(WorldRank(0));
    world.insert_resource(WorldSize(1));
    world.insert_resource(InputParameters {
        paths: vec![file.into()],
        ..Default::default()
    });
    world.insert_non_send_resource(InputDatasetDescriptor::<T>::new(
        DatasetDescriptor::default_for::<T>(),
        DatasetShape::OneDimensional,
    ));
    run_system_on_world(world, read_dataset_system::<T>);
}
