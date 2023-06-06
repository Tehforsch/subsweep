use bevy::prelude::Res;
use hdf5::File;
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::components::Temperature;
use raxiom::io::input::read_dataset_for_file;
use raxiom::io::DatasetShape;
use raxiom::io::DefaultUnitReader;

use super::unit_reader::make_descriptor;
use super::unit_reader::read_vec;
use super::Parameters;

pub fn remap_abundances_and_energies_system(parameters: Res<Parameters>) {
    let unit_reader = DefaultUnitReader;
    // TODO
    let scale_factor = 1.0;
    let file = match &parameters.remap_from {
        Some(file) => file,
        None => return,
    };
    let file = File::open(file).unwrap();
    let descriptor = &make_descriptor::<Position, _>(
        &unit_reader,
        "position",
        DatasetShape::TwoDimensional(read_vec),
    );
    let position = read_dataset_for_file(descriptor, &file);
    let descriptor = &make_descriptor::<IonizedHydrogenFraction, _>(
        &unit_reader,
        "ionized_hydrogen_fraction",
        DatasetShape::OneDimensional,
    );
    let ionized_hydrogen_fraction = read_dataset_for_file(descriptor, &file);
    let descriptor = &make_descriptor::<Temperature, _>(
        &unit_reader,
        "temperature",
        DatasetShape::OneDimensional,
    );
    let temperature = read_dataset_for_file(descriptor, &file);
    dbg!(position, ionized_hydrogen_fraction, temperature);
}
