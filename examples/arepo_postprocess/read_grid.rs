use std::path::Path;

use bevy::prelude::Component;
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
use raxiom::io::input::read_dataset_for_file;
use raxiom::io::input::DatasetInputPlugin;
use raxiom::io::to_dataset::ToDataset;
use raxiom::io::unit_reader::IdReader;
use raxiom::io::DatasetDescriptor;
use raxiom::io::DatasetShape;
use raxiom::io::InputDatasetDescriptor;
use raxiom::prelude::ParticleId;
use raxiom::prelude::Particles;
use raxiom::prelude::RaxiomPlugin;
use raxiom::prelude::Simulation;
use raxiom::simulation_plugin::StartupStages;
use raxiom::sweep::grid::Cell;
use raxiom::units;
use raxiom::units::Volume;
use raxiom::units::NONE;

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
)]
#[name = "UniqueParticleId"]
#[repr(transparent)]
pub struct UniqueParticleId(pub u64);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "Mass"]
#[repr(transparent)]
pub struct Mass(pub units::Mass);

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "Area"]
#[repr(transparent)]
pub struct Area(pub units::Area);

impl ToDataset for UniqueParticleId {
    fn dimension() -> raxiom::units::Dimension {
        NONE
    }

    fn convert_base_units(self, _factor: f64) -> Self {
        self
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
}

fn read_connections(file: &Path, cosmology: &Cosmology) {
    let unit_reader = ArepoUnitReader::new(cosmology.clone());
    let file = File::open(file)
        .unwrap_or_else(|_| panic!("Failed to open grid file: {}", file.to_str().unwrap()));
    let descriptor =
        &make_descriptor::<UniqueParticleId, _>(&IdReader, "Id1", DatasetShape::OneDimensional);
    let ids1 = read_dataset_for_file(descriptor, &file);
    let descriptor =
        &make_descriptor::<UniqueParticleId, _>(&IdReader, "Id2", DatasetShape::OneDimensional);
    let ids2 = read_dataset_for_file(descriptor, &file);
    let descriptor =
        &make_descriptor::<Area, _>(&unit_reader, "Area", DatasetShape::OneDimensional);
    let areas = read_dataset_for_file(descriptor, &file);
}

impl Constructor {
    fn new<'a>(particle_ids: Vec<(ParticleId, UniqueParticleId, Volume)>) -> Self {
        let map = particle_ids
            .iter()
            .map(|(id1, id2, _)| (id1.clone(), id2.clone()))
            .collect();
        let cells = particle_ids
            .iter()
            .map(|(_, _, volume)| Cell {
                neighbours: vec![],
                size: volume.cbrt(),
                volume: *volume,
            })
            .collect();
        Self { map, cells }
    }
}

fn read_grid_system(
    p: Particles<(&ParticleId, &UniqueParticleId, &Mass, &Density)>,
    parameters: Res<Parameters>,
    cosmology: Res<Cosmology>,
) {
    let mut constructor = Constructor::new(
        p.iter()
            .map(|(id1, id2, mass, density)| (id1.clone(), id2.clone(), **mass / **density))
            .collect(),
    );
    let grid_file = if let GridParameters::Read(ref path) = parameters.grid {
        path.clone()
    } else {
        unreachable!()
    };
    let connections = read_connections(&grid_file, &cosmology);
}
