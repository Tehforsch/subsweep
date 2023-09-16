use std::iter::once;
use std::path::Path;
use std::path::PathBuf;

use bevy_ecs::prelude::Entity;
use bevy_ecs::prelude::Res;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use log::debug;
use log::info;
use mpi::traits::Equivalence;
use subsweep::communication::communicator::Communicator;
use subsweep::communication::CommunicatedOption;
use subsweep::communication::DataByRank;
use subsweep::communication::ExchangeCommunicator;
use subsweep::communication::Identified;
use subsweep::communication::SizedCommunicator;
use subsweep::components::IonizedHydrogenFraction;
use subsweep::components::Position;
use subsweep::components::Temperature;
use subsweep::cosmology::LittleH;
use subsweep::cosmology::ScaleFactor;
use subsweep::domain::DecompositionState;
use subsweep::domain::IntoKey;
use subsweep::hash_map::HashMap;
use subsweep::io::input::attribute::read_attribute;
use subsweep::io::input::get_file_or_all_hdf5_files_in_path_if_dir;
use subsweep::io::input::Reader;
use subsweep::io::DatasetShape;
use subsweep::io::DefaultUnitReader;
use subsweep::parameters::Cosmology;
use subsweep::prelude::Extent;
use subsweep::prelude::Float;
use subsweep::prelude::Particles;
use subsweep::prelude::SimulationBox;
use subsweep::units::Dimension;
use subsweep::units::Dimensionless;
use subsweep::units::Length;
use subsweep::units::VecLength;

use super::unit_reader::make_descriptor;
use super::Parameters;

type Tree = KdTree<Float, 3>;

const CHUNK_SIZE: usize = 1000000;

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

#[derive(Equivalence, Clone, Debug)]
struct FullRemapData {
    position: Position,
    temperature: Temperature,
    ionized_hydrogen_fraction: IonizedHydrogenFraction,
}

impl From<FullRemapData> for RemapData {
    fn from(data: FullRemapData) -> Self {
        RemapData {
            temperature: data.temperature,
            ionized_hydrogen_fraction: data.ionized_hydrogen_fraction,
        }
    }
}

fn read_remap_data(files: Vec<PathBuf>, cosmology: &Cosmology) -> Vec<FullRemapData> {
    let reader = Reader::split_between_ranks(files.iter());
    let unit_reader = DefaultUnitReader;
    let descriptor =
        make_descriptor::<Position, _>(&unit_reader, "position", DatasetShape::OneDimensional);
    let position = reader.read_dataset(descriptor);
    let descriptor = make_descriptor::<IonizedHydrogenFraction, _>(
        &unit_reader,
        "ionized_hydrogen_fraction",
        DatasetShape::OneDimensional,
    );
    let ionized_hydrogen_fraction = reader.read_dataset(descriptor);
    let descriptor = make_descriptor::<Temperature, _>(
        &unit_reader,
        "temperature",
        DatasetShape::OneDimensional,
    );
    let temperature = reader.read_dataset(descriptor);
    let scale_factor = read_attribute::<ScaleFactor>(&files[0]);
    let little_h = read_attribute::<LittleH>(&files[0]);
    let remap_cosmology = Cosmology::Cosmological {
        a: *scale_factor.0,
        h: *little_h.0,
        params: None,
    };
    let factor = get_scale_factor_difference(Length::dimension(), cosmology, &remap_cosmology);
    position
        .zip(ionized_hydrogen_fraction)
        .zip(temperature)
        .map(|((position, ionized_hydrogen_fraction), temperature)| {
            let position = Position(*position * factor);
            FullRemapData {
                position,
                ionized_hydrogen_fraction,
                temperature,
            }
        })
        .collect()
}

struct Remapper<'a, 'w, 's> {
    data: Vec<FullRemapData>,
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
        box_: &SimulationBox,
        decomposition: &DecompositionState,
    ) -> Self {
        let data = read_remap_data(files, cosmology);
        let comm1 = ExchangeCommunicator::<Identified<SearchRequest>>::new();
        let comm2 = ExchangeCommunicator::<Identified<SearchReply>>::new();
        let data = exchange_according_to_domain_decomposition(data, box_, decomposition);
        let tree: Tree = (&data
            .iter()
            .map(|d| pos_to_tree_coord(&d.position))
            .collect::<Vec<_>>())
            .into();
        let extents = exchange_extents(&data);
        Remapper::<'a, 'w, 's> {
            data,
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

    /// Construct the outgoing requests by only sending requests to
    /// those ranks that could have a particle that is closer to the
    /// particle than the distance to the locally closest particle.
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
                if self.closer_particle_could_exist_on_other_rank(
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

    fn closer_particle_could_exist_on_other_rank(
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
            data: self.data[index].clone().into(),
        }
    }
}

/// Exchange particles in the remap file according to the
/// (already existing) domain decomposition of the local particles.
fn exchange_according_to_domain_decomposition(
    data: Vec<FullRemapData>,
    box_: &SimulationBox,
    decomposition: &DecompositionState,
) -> Vec<FullRemapData> {
    let mut comm = ExchangeCommunicator::<FullRemapData>::new();
    let mut outgoing_data: DataByRank<Vec<FullRemapData>> =
        DataByRank::same_for_all_ranks_in_communicator(vec![], &comm);
    let this_rank = comm.rank();
    let world_size = comm.size();
    for d in data {
        let key = d.position.into_key(box_);
        let mut rank = decomposition.get_owning_rank(key);
        // Sometimes the decomposition will return ranks outside of the range,
        // because of lookup points outside the simulation box. Just keep these
        // on the local rank.
        if rank as usize >= world_size {
            rank = this_rank;
        }
        outgoing_data.get_mut(&rank).unwrap().push(d);
    }
    let remaining = outgoing_data.remove(&this_rank).unwrap();
    let incoming = comm.exchange_all(outgoing_data);
    incoming
        .into_iter()
        .chain(once((this_rank, remaining)))
        .flat_map(|(_, data)| data)
        .collect()
}

fn exchange_extents(data: &[FullRemapData]) -> DataByRank<Extent> {
    let mut extent_communicator = Communicator::<CommunicatedOption<Extent>>::new();
    let extent = Extent::from_positions(data.iter().map(|x| &*x.position));
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

pub fn remap_abundances_and_energies_system(
    parameters: Res<Parameters>,
    cosmology: Res<Cosmology>,
    box_: Res<SimulationBox>,
    decomposition: Res<DecompositionState>,
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
    let mut remapper = Remapper::new(files, &cosmology, &mut particles, &box_, &decomposition);
    remapper.remap();
}
