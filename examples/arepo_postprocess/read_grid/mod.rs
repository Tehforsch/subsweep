mod remote_id_cache;

use std::path::Path;

use bevy::prelude::info;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use bimap::BiMap;
use derive_custom::Named;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use hdf5::File;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use raxiom::components::Density;
use raxiom::cosmology::Cosmology;
use raxiom::hash_map::HashMap;
use raxiom::io::input::read_dataset_for_file;
use raxiom::io::input::DatasetInputPlugin;
use raxiom::io::to_dataset::ToDataset;
use raxiom::io::unit_reader::IdReader;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::prelude::Float;
use raxiom::prelude::ParticleId;
use raxiom::prelude::Particles;
use raxiom::prelude::RaxiomPlugin;
use raxiom::prelude::Simulation;
use raxiom::simulation_plugin::StartupStages;
use raxiom::sweep::grid::Cell;
use raxiom::sweep::grid::Face;
use raxiom::sweep::grid::ParticleType;
use raxiom::sweep::grid::PeriodicNeighbour;
use raxiom::sweep::grid::RemoteNeighbour;
use raxiom::sweep::grid::RemotePeriodicNeighbour;
use raxiom::sweep::SweepParameters;
use raxiom::units;
use raxiom::units::MVec;
use raxiom::units::VecDimensionless;
use raxiom::units::Volume;
use raxiom::units::NONE;

use self::remote_id_cache::RemoteIdCache;
use crate::unit_reader::make_descriptor;
use crate::unit_reader::ArepoUnitReader;
use crate::GridParameters;
use crate::Parameters;

#[derive(Named)]
pub struct ReadSweepGridPlugin;

#[derive(
    H5Type,
    Component,
    Debug,
    Clone,
    Equivalence,
    Deref,
    DerefMut,
    From,
    Default,
    Named,
    PartialEq,
    Eq,
    Hash,
    Copy,
)]
#[name = "UniqueParticleId"]
#[repr(transparent)]
pub struct UniqueParticleId(pub u64);

#[derive(
    H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named, Copy,
)]
#[name = "ConnectionTypeInt"]
#[repr(transparent)]
pub struct ConnectionTypeInt(pub i32);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "Mass"]
#[repr(transparent)]
pub struct Mass(pub units::Mass);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "Area"]
#[repr(transparent)]
pub struct Area(pub units::Area);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "FaceNormal"]
#[repr(transparent)]
pub struct FaceNormal(pub units::VecDimensionless);

impl ToDataset for UniqueParticleId {
    fn dimension() -> raxiom::units::Dimension {
        NONE
    }

    fn convert_base_units(self, _factor: f64) -> Self {
        self
    }
}

impl ToDataset for ConnectionTypeInt {
    fn dimension() -> raxiom::units::Dimension {
        NONE
    }

    fn convert_base_units(self, _factor: f64) -> Self {
        self
    }
}

#[derive(Debug)]
struct ConnectionType {
    periodic1: bool,
    periodic2: bool,
    valid: bool,
}

fn periodic_and_boundary_flags_from_bits(bits: i32) -> (bool, bool) {
    let periodic = bits & 1 > 0;
    let boundary = bits & 2 > 0;
    (periodic, boundary)
}

impl From<ConnectionTypeInt> for ConnectionType {
    fn from(value: ConnectionTypeInt) -> Self {
        let (periodic1, boundary1) = periodic_and_boundary_flags_from_bits(*value & (1 + 2));
        let (periodic2, boundary2) = periodic_and_boundary_flags_from_bits((*value & (4 + 8)) >> 2);
        ConnectionType {
            periodic1,
            periodic2,
            valid: !(boundary1 || boundary2 || (periodic1 && periodic2)),
        }
    }
}

impl RaxiomPlugin for ReadSweepGridPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let cosmology = sim.get_parameters::<Cosmology>().clone();
        let unit_reader = Box::new(ArepoUnitReader::new(cosmology));
        sim.add_plugin(DatasetInputPlugin::<UniqueParticleId>::from_descriptor(
            InputDatasetDescriptor::<UniqueParticleId> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/ParticleIDs".into(),
                    unit_reader: Box::new(IdReader),
                },
                ..Default::default()
            },
        ))
        .add_plugin(DatasetInputPlugin::<Mass>::from_descriptor(
            InputDatasetDescriptor::<Mass> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/Masses".into(),
                    unit_reader: unit_reader,
                },
                ..Default::default()
            },
        ))
        .add_component_no_io::<UniqueParticleId>()
        .add_component_no_io::<Mass>()
        .add_startup_system_to_stage(StartupStages::InsertGrid, read_grid_system);
    }
}

struct Constructor {
    map: BiMap<ParticleId, UniqueParticleId>,
    cells: Vec<Cell>,
    unique_particle_id_to_index: HashMap<UniqueParticleId, usize>,
    allow_periodic: bool,
    remote_id_cache: RemoteIdCache,
}

#[derive(Debug)]
struct Connection {
    area: Area,
    normal: FaceNormal,
    id1: UniqueParticleId,
    id2: UniqueParticleId,
    type_: ConnectionType,
}

fn read_normal(data: &[Float]) -> FaceNormal {
    FaceNormal(VecDimensionless::new_unchecked(MVec::new(
        data[0], data[1], data[2],
    )))
}

fn read_connection_data(file: &Path, cosmology: &Cosmology) -> impl Iterator<Item = Connection> {
    let unit_reader = ArepoUnitReader::new(cosmology.clone());
    let file = File::open(file)
        .unwrap_or_else(|_| panic!("Failed to open grid file: {}", file.to_str().unwrap()));
    let descriptor =
        &make_descriptor::<UniqueParticleId, _>(&IdReader, "Id1", DatasetShape::OneDimensional);
    let ids1 = read_dataset_for_file(descriptor, &file);
    let descriptor =
        &make_descriptor::<UniqueParticleId, _>(&IdReader, "Id2", DatasetShape::OneDimensional);
    let ids2 = read_dataset_for_file(descriptor, &file);
    let descriptor = &make_descriptor::<ConnectionTypeInt, _>(
        &IdReader,
        "ConnectionType",
        DatasetShape::OneDimensional,
    );
    let connection_types = read_dataset_for_file(descriptor, &file);
    let descriptor =
        &make_descriptor::<Area, _>(&unit_reader, "Area", DatasetShape::OneDimensional);
    let areas = read_dataset_for_file(descriptor, &file);
    let descriptor = &make_descriptor::<FaceNormal, _>(
        &unit_reader,
        "Normal",
        DatasetShape::TwoDimensional(read_normal),
    );
    let normals = read_dataset_for_file(descriptor, &file);
    ids1.into_iter()
        .zip(
            ids2.into_iter().zip(
                connection_types
                    .into_iter()
                    .zip(areas.into_iter().zip(normals.into_iter())),
            ),
        )
        .map(
            |(id1, (id2, (connection_type, (area, normal))))| Connection {
                id1,
                id2,
                type_: connection_type.into(),
                area,
                normal,
            },
        )
}

impl Constructor {
    fn new<'a>(
        particle_ids: Vec<(ParticleId, UniqueParticleId, Volume)>,
        allow_periodic: bool,
    ) -> Self {
        let map = particle_ids
            .iter()
            .map(|(id1, id2, _)| (id1.clone(), id2.clone()))
            .collect();
        let unique_particle_id_to_index = particle_ids
            .iter()
            .enumerate()
            .map(|(i, (_, id, _))| (id.clone(), i))
            .collect();
        let cells = particle_ids
            .iter()
            .map(|(_, _, volume)| Cell {
                neighbours: vec![],
                size: volume.cbrt(),
                volume: *volume,
            })
            .collect();
        Self {
            map,
            cells,
            unique_particle_id_to_index,
            allow_periodic,
            remote_id_cache: RemoteIdCache::new(),
        }
    }

    fn get_cell_by_id(&mut self, id: UniqueParticleId) -> &mut Cell {
        &mut self.cells[self.unique_particle_id_to_index[&id]]
    }

    fn add_connections(&mut self, grid_file: &std::path::PathBuf, cosmology: &Cosmology) {
        let connections = read_connection_data(&grid_file, &cosmology);
        for connection in connections {
            if !connection.type_.valid {
                continue;
            }
            let face1 = Face {
                area: *connection.area,
                normal: *connection.normal,
            };
            let face2 = Face {
                area: *connection.area,
                normal: -*connection.normal,
            };
            let ptype1 = self.get_particle_type(connection.id1, connection.type_.periodic1);
            let ptype2 = self.get_particle_type(connection.id2, connection.type_.periodic2);
            if ptype1.is_local() {
                self.get_cell_by_id(connection.id1)
                    .neighbours
                    .push((face2, ptype2));
            }
            if ptype2.is_local() {
                self.get_cell_by_id(connection.id2)
                    .neighbours
                    .push((face1, ptype1));
            }
        }
    }

    fn get_particle_type(&self, id: UniqueParticleId, is_periodic: bool) -> ParticleType {
        let local_id = self.map.get_by_right(&id);
        let is_local = local_id.is_some();
        match (is_local, is_periodic) {
            (true, false) => ParticleType::Local(*local_id.unwrap()),
            (true, true) => {
                if self.allow_periodic {
                    // Find out periodic wrap type. Probably safe to
                    // construct garbage but then again, I can let
                    // Arepo tell me by doing some bit mangling
                    let periodic_neighbour = PeriodicNeighbour {
                        id: *local_id.unwrap(),
                        periodic_wrap_type: todo!(),
                    };
                    ParticleType::LocalPeriodic(periodic_neighbour)
                } else {
                    ParticleType::Boundary
                }
            }
            (false, true) => {
                let (rank, id) = self.remote_id_cache.find(id);
                ParticleType::Remote(RemoteNeighbour { id, rank })
            }
            (false, false) => {
                let (rank, id) = self.remote_id_cache.find(id);
                if self.allow_periodic {
                    // Find out periodic wrap type. Probably safe to
                    // construct garbage but then again, I can let
                    // Arepo tell me by doing some bit mangling
                    let remote_periodic_neighbour = RemotePeriodicNeighbour {
                        id,
                        rank,
                        periodic_wrap_type: todo!(),
                    };
                    ParticleType::RemotePeriodic(remote_periodic_neighbour)
                } else {
                    ParticleType::Boundary
                }
            }
        }
    }
}

fn read_grid_system(
    mut commands: Commands,
    p: Particles<(Entity, &ParticleId, &UniqueParticleId, &Mass, &Density)>,
    parameters: Res<Parameters>,
    sweep_parameters: Res<SweepParameters>,
    cosmology: Res<Cosmology>,
) {
    let grid_file = if let GridParameters::Read(ref path) = parameters.grid {
        path.clone()
    } else {
        unreachable!()
    };
    info!("Reading grid from {:?}", grid_file);
    let mut constructor = Constructor::new(
        p.iter()
            .map(|(_, id1, id2, mass, density)| (id1.clone(), id2.clone(), **mass / **density))
            .collect(),
        sweep_parameters.periodic,
    );
    constructor.add_connections(&grid_file, &cosmology);
    for ((entity, _, _, _, _), cell) in p.iter().zip(constructor.cells) {
        commands.entity(entity).insert(cell);
    }
}