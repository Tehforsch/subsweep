use std::path::Path;

use bevy::prelude::*;
use derive_custom::raxiom_parameters;
use ordered_float::OrderedFloat;
use raxiom::components;
use raxiom::components::Density;
use raxiom::components::Position;
use raxiom::components::Source;
use raxiom::parameters::Cosmology;
use raxiom::parameters::SimulationParameters;
use raxiom::prelude::Extent;
use raxiom::prelude::LocalParticle;
use raxiom::prelude::Particles;
use raxiom::prelude::Simulation;
use raxiom::prelude::SimulationBox;
use raxiom::prelude::SimulationBuilder;
use raxiom::prelude::StartupStages;
use raxiom::prelude::WorldRank;
use raxiom::prelude::WorldSize;
use raxiom::sweep::grid::init_cartesian_grid_system;
use raxiom::sweep::grid::NumCellsSpec;
use raxiom::sweep::SweepPlugin;
use raxiom::units::Dimensionless;
use raxiom::units::Length;
use raxiom::units::Mass;
use raxiom::units::PhotonFlux;
use raxiom::units::PhotonRate;
use raxiom::units::SourceRate;
use raxiom::units::Temperature;
use raxiom::units::Time;
use raxiom::units::VecLength;
use raxiom::units::Volume;

const BOX_SIZE_MEGAPARSEC: f64 = 1.0;

#[raxiom_parameters("1d")]
struct Params {
    num_particles: usize,
    photon_flux: PhotonFlux,
}

fn main() {
    let mut sim = setup_sweep_sim();
    sim.run();
}

fn setup_sweep_sim() -> Simulation {
    let mut sim = SimulationBuilder::new();
    let mut sim = sim
        .write_output(true)
        .read_initial_conditions(false)
        .require_parameter_file(true)
        .verbosity(2)
        .parameter_file_path(&Path::new("params.yml").to_owned())
        .build();
    let params = sim.add_parameter_type_and_get_result::<Params>().clone();
    add_box_size(&mut sim, params.num_particles);
    add_grid(&mut sim, params.num_particles);
    sim.write_output(true)
        .add_parameters_explicitly(Cosmology::NonCosmological)
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::megayears(1000.0)),
        })
        .add_startup_system_to_stage(
            StartupStages::InsertComponentsAfterGrid,
            initialize_sweep_components_system,
        )
        .add_startup_system_to_stage(StartupStages::InsertComponentsAfterGrid, add_source_system)
        .add_plugin(SweepPlugin);
    sim
}

fn add_box_size(sim: &mut Simulation, num_particles: usize) {
    let cell_size = Length::megaparsec(BOX_SIZE_MEGAPARSEC) / num_particles as f64;
    let min = VecLength::zero();
    let max = VecLength::new(cell_size * num_particles as f64, cell_size, cell_size);
    let extent = Extent {
        min,
        max,
        center: (min + max) / 2.0,
    };
    let box_size = SimulationBox::new(extent);
    sim.insert_resource(box_size);
}

fn add_grid(sim: &mut Simulation, num_particles: usize) {
    let cell_size = Length::megaparsec(BOX_SIZE_MEGAPARSEC) / num_particles as f64;
    let grid_setup = move |commands: Commands,
                           box_size: Res<SimulationBox>,
                           world_size: Res<WorldSize>,
                           world_rank: Res<WorldRank>| {
        init_cartesian_grid_system(
            commands,
            box_size,
            NumCellsSpec::CellSize(cell_size),
            world_size,
            world_rank,
            false,
        )
    };
    sim.add_startup_system(grid_setup);
}

fn add_source_system(
    mut commands: Commands,
    particles: Particles<(Entity, &Position)>,
    params: Res<Params>,
) {
    let most_left = particles
        .iter()
        .min_by_key(|(_, pos)| OrderedFloat(pos.x().value_unchecked()))
        .unwrap();

    for (entity, _) in particles.iter() {
        let source = if entity != most_left.0 {
            SourceRate::zero()
        } else {
            let num_particles = particles.iter().count();
            let cell_size = Length::megaparsec(BOX_SIZE_MEGAPARSEC) / num_particles as f64;
            let photons_per_second = params.photon_flux * cell_size.squared();
            photons_per_second
        };
        commands.entity(entity).insert(Source(source));
    }
}

pub fn initialize_sweep_components_system(
    mut commands: Commands,
    local_particles: Query<Entity, With<LocalParticle>>,
) {
    for entity in local_particles.iter() {
        commands.entity(entity).insert((
            Density(Mass::grams(1.0e-27) / Volume::cubic_centimeters(1.0)),
            components::IonizedHydrogenFraction(Dimensionless::dimensionless(1e-10)),
            components::Temperature(Temperature::kelvins(1000.0)),
            components::PhotonRate(PhotonRate::zero()),
        ));
    }
}
