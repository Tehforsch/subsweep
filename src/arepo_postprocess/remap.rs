use std::fs;
use std::path::Path;
use std::path::PathBuf;

use bevy::prelude::debug;
use bevy::prelude::info;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use mpi::traits::Equivalence;
use raxiom::communication::communicator::Communicator;
use raxiom::communication::DataByRank;
use raxiom::communication::ExchangeCommunicator;
use raxiom::communication::Identified;
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::components::Temperature;
use raxiom::hash_map::HashMap;
use raxiom::io::input::Reader;
use raxiom::io::DatasetShape;
use raxiom::io::DefaultUnitReader;
use raxiom::prelude::Float;
use raxiom::prelude::Particles;
use raxiom::units::Length;
use raxiom::units::VecLength;

use super::unit_reader::make_descriptor;
use super::Parameters;

type Tree = KdTree<Float, 3>;

#[derive(Equivalence, Clone, Debug)]
struct SearchRequest {
    pos: Position,
}

#[derive(Equivalence, Clone, Debug)]
struct SearchReply {
    distance: Length,
    data: RemapData,
}

#[derive(Equivalence, Clone, Debug)]
struct RemapData {
    temperature: Temperature,
    ionized_hydrogen_fraction: IonizedHydrogenFraction,
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

pub fn remap_abundances_and_energies_system(
    parameters: Res<Parameters>,
    mut particles: Particles<(
        Entity,
        &Position,
        &mut Temperature,
        &mut IonizedHydrogenFraction,
    )>,
) {
    const CHUNK_SIZE: usize = 50000;
    let files = match &parameters.remap_from {
        Some(file) => get_files(file),
        None => return,
    };
    info!("Remapping abundances and temperatures.");

    let (position, ionized_hydrogen_fraction, temperature) = read_remap_data(files);
    let requests: Vec<_> = particles
        .iter()
        .map(|(entity, pos, _, _)| Identified::new(entity, SearchRequest { pos: pos.clone() }))
        .collect();
    let num_chunks = global_num_chunks(requests.len(), CHUNK_SIZE);
    let mut chunk_iter = requests.chunks(CHUNK_SIZE);
    let tree: Tree = (&position
        .iter()
        .map(|pos| pos_to_tree_coord(pos))
        .collect::<Vec<_>>())
        .into();
    for _ in 0..num_chunks {
        let chunk = chunk_iter.next().unwrap_or(&[]);
        exchange_request_chunk(
            &mut particles,
            &ionized_hydrogen_fraction,
            &temperature,
            &tree,
            chunk,
        );
    }
    debug!("Finished remapping.");
}

fn exchange_request_chunk(
    particles: &mut Particles<(
        Entity,
        &Position,
        &mut Temperature,
        &mut IonizedHydrogenFraction,
    )>,
    ionized_hydrogen_fraction: &[IonizedHydrogenFraction],
    temperature: &[Temperature],
    tree: &Tree,
    chunk: &[Identified<SearchRequest>],
) {
    let mut comm = ExchangeCommunicator::<Identified<SearchRequest>>::new();
    let outgoing =
        DataByRank::same_for_all_ranks_in_communicator(chunk.iter().cloned().collect(), &comm);
    let incoming = comm.exchange_all(outgoing);
    let mut outgoing: DataByRank<Vec<Identified<SearchReply>>> =
        DataByRank::from_communicator(&comm);
    for (rank, requests) in incoming {
        for request in requests {
            let tree_coord = pos_to_tree_coord(&request.data.pos);
            let (distance, index) = tree.nearest_one(&tree_coord, &squared_euclidean);
            let reply = Identified::new(
                request.entity(),
                SearchReply {
                    distance: Length::new_unchecked(distance),
                    data: RemapData {
                        temperature: temperature[index].clone(),
                        ionized_hydrogen_fraction: ionized_hydrogen_fraction[index].clone(),
                    },
                },
            );
            outgoing[rank].push(reply);
        }
    }
    let mut comm = ExchangeCommunicator::<Identified<SearchReply>>::new();
    let incoming = comm.exchange_all(outgoing);
    let mut distance_map = HashMap::default();
    let mut value_map = HashMap::default();
    let infinity = Length::new_unchecked(f64::INFINITY);
    for (_, replies) in incoming {
        for reply in replies {
            let entity = reply.entity();
            let old_distance = distance_map.get(&entity).unwrap_or(&infinity);
            if reply.data.distance < *old_distance {
                value_map.insert(entity, reply.data.data);
                distance_map.insert(entity, reply.data.distance);
            }
        }
    }
    for (entity, data) in value_map.into_iter() {
        let (_, _, mut temp, mut ion_frac) = particles.get_mut(entity).unwrap();
        remap_from(&mut temp, &mut ion_frac, data);
    }
}

fn remap_from(temp: &mut Temperature, ion_frac: &mut IonizedHydrogenFraction, data: RemapData) {
    **temp = (*temp).max(*data.temperature);
    **ion_frac = (*ion_frac).max(*data.ionized_hydrogen_fraction);
}

fn pos_to_tree_coord(pos: &VecLength) -> [f64; 3] {
    [
        pos.x().value_unchecked(),
        pos.y().value_unchecked(),
        pos.z().value_unchecked(),
    ]
}

fn global_num_chunks(num_elements: usize, chunk_size: usize) -> usize {
    let mut comm: Communicator<usize> = Communicator::new();
    let num_chunks = div_ceil(num_elements, chunk_size);
    comm.all_gather_max(&num_chunks).unwrap()
}

fn div_ceil(x: usize, y: usize) -> usize {
    (x / y) + if x.rem_euclid(y) > 0 { 1 } else { 0 }
}
