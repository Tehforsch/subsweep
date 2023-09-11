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
use raxiom::communication::CommunicatedOption;
use raxiom::communication::DataByRank;
use raxiom::communication::ExchangeCommunicator;
use raxiom::communication::Identified;
use raxiom::communication::SizedCommunicator;
use raxiom::components::IonizedHydrogenFraction;
use raxiom::components::Position;
use raxiom::components::Temperature;
use raxiom::cosmology::LittleH;
use raxiom::cosmology::ScaleFactor;
use raxiom::hash_map::HashMap;
use raxiom::io::input::attribute::read_attribute;
use raxiom::io::input::get_file_or_all_hdf5_files_in_path_if_dir;
use raxiom::io::input::Reader;
use raxiom::io::DatasetShape;
use raxiom::io::DefaultUnitReader;
use raxiom::parameters::Cosmology;
use raxiom::prelude::Extent;
use raxiom::prelude::Float;
use raxiom::prelude::Particles;
use raxiom::units::Dimension;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::VecLength;

use super::unit_reader::make_descriptor;
use super::Parameters;

type Tree = KdTree<Float, 3>;

const CHUNK_SIZE: usize = 50000;

#[derive(Equivalence, Clone, Debug)]
struct SearchRequest {
    pos: Position,
}

#[derive(Equivalence, Clone, Debug)]
struct SearchReply {
    squared_distance: f64,
    data: RemapData,
}

#[derive(Equivalence, Clone, Debug)]
struct RemapData {
    temperature: Temperature,
    ionized_hydrogen_fraction: IonizedHydrogenFraction,
}

fn read_remap_data(
    files: Vec<PathBuf>,
) -> (
    Vec<Position>,
    Vec<IonizedHydrogenFraction>,
    Vec<Temperature>,
    Cosmology,
) {
    let reader = Reader::split_between_ranks(files.iter());
    let unit_reader = DefaultUnitReader;
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
    let scale_factor = read_attribute::<ScaleFactor>(&files[0]);
    let little_h = read_attribute::<LittleH>(&files[0]);
    let cosmology = Cosmology::Cosmological {
        a: *scale_factor.0,
        h: *little_h.0,
        params: None,
    };
    (position, ionized_hydrogen_fraction, temperature, cosmology)
}

struct Remapper<'a, 'w, 's> {
    ionized_hydrogen_fraction: Vec<IonizedHydrogenFraction>,
    temperature: Vec<Temperature>,
    tree: Tree,
    extents: DataByRank<Extent>,
    particles: &'a mut Particles<
        'w,
        's,
        (
            Entity,
            &'static Position,
            &'static mut Temperature,
            &'static mut IonizedHydrogenFraction,
        ),
    >,
    comm1: ExchangeCommunicator<Identified<SearchRequest>>,
    comm2: ExchangeCommunicator<Identified<SearchReply>>,
}

impl<'a, 'w, 's> Remapper<'a, 'w, 's> {
    fn new(
        files: Vec<PathBuf>,
        cosmology: &Cosmology,
        particles: &'a mut Particles<
            'w,
            's,
            (
                Entity,
                &'static Position,
                &'static mut Temperature,
                &'static mut IonizedHydrogenFraction,
            ),
        >,
    ) -> Self {
        let (position, ionized_hydrogen_fraction, temperature, remap_cosmology) =
            read_remap_data(files);
        let factor = get_scale_factor_difference(Length::dimension(), cosmology, &remap_cosmology);
        let tree: Tree = (&position
            .iter()
            .map(|pos| pos_to_tree_coord(&(**pos * factor)))
            .collect::<Vec<_>>())
            .into();
        let extents = exchange_extents(&position);
        let comm1 = ExchangeCommunicator::<Identified<SearchRequest>>::new();
        let comm2 = ExchangeCommunicator::<Identified<SearchReply>>::new();
        Remapper::<'a, 'w, 's> {
            ionized_hydrogen_fraction,
            temperature,
            tree,
            particles,
            extents,
            comm1,
            comm2,
        }
    }

    fn remap(&mut self) {
        let requests: Vec<_> = self
            .particles
            .iter()
            .map(|(entity, pos, _, _)| Identified::new(entity, SearchRequest { pos: pos.clone() }))
            .collect();
        let num_chunks = global_num_chunks(requests.len(), CHUNK_SIZE);
        let mut chunk_iter = requests.chunks(CHUNK_SIZE);
        for _ in 0..num_chunks {
            let chunk = chunk_iter.next().unwrap_or(&[]);
            self.exchange_request_chunk(chunk);
        }
        debug!("Finished remapping.");
    }

    fn exchange_request_chunk(&mut self, chunk: &[Identified<SearchRequest>]) {
        let mut closest_map: HashMap<_, _> = chunk
            .iter()
            .map(|request| (request.entity(), self.get_reply(&request.data)))
            .collect();
        let outgoing = self.get_outgoing_requests(&closest_map, chunk);
        let incoming = self.comm1.exchange_all(outgoing);
        let mut outgoing: DataByRank<Vec<Identified<SearchReply>>> =
            DataByRank::from_communicator(&self.comm2);
        for (rank, requests) in incoming {
            for request in requests {
                let reply = Identified::new(request.entity(), self.get_reply(&request.data));
                outgoing[rank].push(reply);
            }
        }
        let incoming = self.comm2.exchange_all(outgoing);
        for (_, replies) in incoming {
            for reply in replies {
                let entity = reply.entity();
                let previously_closest = &closest_map[&entity];
                if previously_closest.squared_distance > reply.data.squared_distance {
                    closest_map.insert(entity, reply.data);
                }
            }
        }
        for (entity, closest) in closest_map.into_iter() {
            let (_, _, mut temp, mut ion_frac) = self.particles.get_mut(entity).unwrap();
            remap_from(&mut temp, &mut ion_frac, closest.data);
        }
    }

    fn get_outgoing_requests(
        &self,
        local_map: &HashMap<Entity, SearchReply>,
        chunk: &[Identified<SearchRequest>],
    ) -> DataByRank<Vec<Identified<SearchRequest>>> {
        let mut outgoing: DataByRank<Vec<Identified<SearchRequest>>> =
            DataByRank::from_communicator(&self.comm1);
        for rank in self.comm1.other_ranks() {
            let extent = &self.extents[rank];
            for request in chunk.iter() {
                let local_squared_distance =
                    local_map.get(&request.entity()).unwrap().squared_distance;
                if self.particle_might_be_closer_to_extent(
                    extent,
                    &request.data,
                    local_squared_distance,
                ) {
                    outgoing.get_mut(&rank).unwrap().push(request.clone());
                }
            }
        }
        outgoing
    }

    fn particle_might_be_closer_to_extent(
        &self,
        extent: &Extent,
        request: &SearchRequest,
        squared_distance: f64,
    ) -> bool {
        let squared_distance_extent = extent
            .squared_distance_to_pos(&request.pos.0)
            .value_unchecked();
        squared_distance_extent < squared_distance
    }

    fn get_reply(&self, request: &SearchRequest) -> SearchReply {
        let tree_coord = pos_to_tree_coord(&request.pos);
        let (squared_distance, index) = self.tree.nearest_one(&tree_coord, &squared_euclidean);
        SearchReply {
            squared_distance,
            data: RemapData {
                temperature: self.temperature[index].clone(),
                ionized_hydrogen_fraction: self.ionized_hydrogen_fraction[index].clone(),
            },
        }
    }
}

fn exchange_extents(position: &[Position]) -> DataByRank<Extent> {
    let mut extent_communicator = Communicator::<CommunicatedOption<Extent>>::new();
    let extent = Extent::from_positions(position.iter().map(|x| &x.0));
    let all_extents = extent_communicator.all_gather(&extent.into());
    all_extents
        .into_iter()
        .enumerate()
        .filter_map(|(i, x)| Option::<_>::from(x).map(|x| (i as i32, x)))
        .collect()
}

fn get_files_of_last_snapshot(path: &Path) -> Vec<PathBuf> {
    let last_snapshot = get_highest_number_snapshot_dir(path);
    get_file_or_all_hdf5_files_in_path_if_dir(&last_snapshot)
}

fn get_highest_number_snapshot_dir(path: &Path) -> PathBuf {
    path.read_dir()
        .unwrap()
        .flat_map(|entry| {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .max_by_key(|snap_folder| {
            snap_folder
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("Unexpected folder in snapshot dir: {snap_folder:?}"))
        })
        .expect("No snapshot folder exists. Failed to remap")
}

pub fn remap_abundances_and_energies_system(
    parameters: Res<Parameters>,
    cosmology: Res<Cosmology>,
    mut particles: Particles<(
        Entity,
        &'static Position,
        &'static mut Temperature,
        &'static mut IonizedHydrogenFraction,
    )>,
) {
    let files = match &parameters.remap_from {
        Some(path) => get_files_of_last_snapshot(path),
        None => return,
    };
    info!("Remapping abundances and temperatures.");
    for file in files.iter() {
        debug!("Remapping from file: {file:?}");
    }
    let mut remapper = Remapper::new(files, &cosmology, &mut particles);
    remapper.remap();
}

fn get_scale_factor_difference(
    dimension: Dimension,
    cosmology: &Cosmology,
    remap_cosmology: &Cosmology,
) -> Dimensionless {
    match cosmology {
        Cosmology::Cosmological { .. } => {
            if let Cosmology::Cosmological { .. } = remap_cosmology {
                (*(cosmology.scale_factor() / remap_cosmology.scale_factor()))
                    .powi(dimension.length)
                    .into()
            } else {
                panic!()
            }
        }
        Cosmology::NonCosmological => {
            if let Cosmology::Cosmological { .. } = remap_cosmology {
                panic!()
            } else {
                1.0.into()
            }
        }
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
