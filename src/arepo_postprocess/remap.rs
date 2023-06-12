use std::fs;
use std::path::Path;
use std::path::PathBuf;

use bevy::prelude::Entity;
use bevy::prelude::Res;
use hdf5::File;
use mpi::traits::Equivalence;
use raxiom::communication::DataByRank;
use raxiom::communication::ExchangeCommunicator;
use raxiom::communication::Identified;
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::components::Temperature;
use raxiom::io::input::Reader;
use raxiom::io::DatasetShape;
use raxiom::io::DefaultUnitReader;
use raxiom::prelude::Particles;

use super::unit_reader::make_descriptor;
use super::unit_reader::read_vec;
use super::Parameters;

fn read_remap_data(
    files: Vec<PathBuf>,
) -> (
    Vec<Position>,
    Vec<IonizedHydrogenFraction>,
    Vec<Temperature>,
) {
    let reader = Reader::split_between_ranks(files.into_iter());
    let unit_reader = DefaultUnitReader;
    // TODO
    let scale_factor = 1.0;
    let descriptor =
        make_descriptor::<Position, _>(&unit_reader, "position", DatasetShape::OneDimensional);
    let position = reader.read_dataset(descriptor).collect();
    let descriptor = make_descriptor::<IonizedHydrogenFraction, _>(
        &unit_reader,
        "ionized_hydrogen_fraction",
        DatasetShape::OneDimensional,
    );
    let ionized_hydrogen_fraction = reader.read_dataset(descriptor).collect();
    let descriptor = make_descriptor::<Temperature, _>(
        &unit_reader,
        "temperature",
        DatasetShape::OneDimensional,
    );
    let temperature = reader.read_dataset(descriptor).collect();
    (position, ionized_hydrogen_fraction, temperature)
}

#[derive(Equivalence, Clone, Debug)]
struct SearchRequest {
    pos: Position,
}

fn get_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path.to_owned()]
    } else {
        fs::read_dir(path)
            .unwrap_or_else(|e| {
                panic!("Error: {e} while trying to read remap path {path:?} as directory")
            })
            .flat_map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                let ext = path.extension()?.to_str()?;
                if path.is_file() && ext == "hdf5" {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn remap_abundances_and_energies_system(
    parameters: Res<Parameters>,
    particles: Particles<(
        Entity,
        &Position,
        &mut Temperature,
        &mut IonizedHydrogenFraction,
    )>,
) {
    const CHUNK_SIZE: usize = 10000;
    let files = match &parameters.remap_from {
        Some(file) => get_files(file),
        None => return,
    };

    let (position, ionized_hydrogen_fraction, temperature) = read_remap_data(files);
    let requests: Vec<_> = particles
        .iter()
        .map(|(entity, pos, _, _)| Identified::new(entity, SearchRequest { pos: pos.clone() }))
        .collect();
    let mut comm = ExchangeCommunicator::<Identified<SearchRequest>>::new();
    for chunk in requests.chunks(CHUNK_SIZE) {
        let outgoing =
            DataByRank::same_for_all_ranks_in_communicator(chunk.iter().cloned().collect(), &comm);
        let incoming = comm.exchange_all(outgoing);
        dbg!(&incoming);
    }
}
