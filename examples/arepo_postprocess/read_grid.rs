use bevy::prelude::Component;
use derive_custom::Named;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::From;
use hdf5::H5Type;
use mpi::traits::Equivalence;
use raxiom::io::input::DatasetInputPlugin;
use raxiom::io::to_dataset::ToDataset;
use raxiom::io::unit_reader::IdReader;
use raxiom::io::DatasetDescriptor;
use raxiom::io::InputDatasetDescriptor;
use raxiom::mpidbg;
use raxiom::prelude::Particles;
use raxiom::prelude::RaxiomPlugin;
use raxiom::prelude::Simulation;
use raxiom::simulation_plugin::SimulationStartupStages;
use raxiom::units::NONE;

#[derive(Named)]
pub struct ReadSweepGridPlugin;

#[derive(H5Type, Component, Debug, Clone, Equivalence, Deref, DerefMut, From, Default, Named)]
#[name = "ArepoParticleId"]
#[repr(transparent)]
pub struct ArepoParticleId(pub u64);

impl ToDataset for ArepoParticleId {
    fn dimension() -> raxiom::units::Dimension {
        NONE
    }

    fn convert_base_units(self, _factor: f64) -> Self {
        self
    }
}

impl RaxiomPlugin for ReadSweepGridPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let id_reader = IdReader;
        sim.add_plugin(DatasetInputPlugin::<ArepoParticleId>::from_descriptor(
            InputDatasetDescriptor::<ArepoParticleId> {
                descriptor: DatasetDescriptor {
                    dataset_name: "PartType0/ParticleIDs".into(),
                    unit_reader: Box::new(id_reader),
                },
                ..Default::default()
            },
        ))
        .add_component_no_io::<ArepoParticleId>()
        .add_startup_system_to_stage(SimulationStartupStages::InsertGrid, print_ids_system);
    }
}

fn print_ids_system(p: Particles<&ArepoParticleId>) {
    mpidbg!(p.iter().count());
}
