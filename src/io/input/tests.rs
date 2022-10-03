use bevy::prelude::Query;
use bevy::prelude::World;

use super::close_file_system;
use super::open_file_system;
use super::read_dataset_system;
use super::InputFiles;
use super::InputParameters;
use super::SpawnedEntities;
use crate::prelude::Mass;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::test_utils::run_system_on_world;
use crate::test_utils::tests_path;
use crate::units::assert_is_close;
use crate::units::{self};

#[test]
fn respect_scale_factor() {
    let mut world = World::new();
    let entity = world.spawn().id();
    world.insert_resource(SpawnedEntities(vec![entity]));
    world.insert_resource(InputFiles(vec![]));
    world.insert_resource(WorldRank(0));
    world.insert_resource(WorldSize(1));
    world.insert_resource(InputParameters {
        paths: vec![tests_path().join("input/respect_scale_factor.hdf5")],
    });
    run_system_on_world(&mut world, open_file_system);
    run_system_on_world(&mut world, read_dataset_system::<Mass>);
    run_system_on_world(&mut world, close_file_system);
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
