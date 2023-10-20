use std::path::Path;

use bevy_ecs::prelude::*;
use derive_custom::subsweep_parameters;
use subsweep::components;
use subsweep::components::Density;
use subsweep::parameters::Cosmology;
use subsweep::prelude::Extent;
use subsweep::prelude::LocalParticle;
use subsweep::prelude::Simulation;
use subsweep::prelude::SimulationBox;
use subsweep::prelude::SimulationBuilder;
use subsweep::prelude::StartupStages;
use subsweep::prelude::WorldRank;
use subsweep::prelude::WorldSize;
use subsweep::source_systems::Source;
use subsweep::source_systems::SourcePlugin;
use subsweep::source_systems::Sources;
use subsweep::sweep::grid::init_cartesian_grid_system;
use subsweep::sweep::grid::NumCellsSpec;
use subsweep::sweep::SweepParameters;
use subsweep::sweep::SweepPlugin;
use subsweep::units::Dimensionless;
use subsweep::units::Length;
use subsweep::units::NumberDensity;
use subsweep::units::PhotonFlux;
use subsweep::units::PhotonRate;
use subsweep::units::Temperature;
use subsweep::units::VecLength;
use subsweep::units::PROTON_MASS;

const BOX_SIZE_MEGAPARSEC: f64 = 1.0;

#[subsweep_parameters("1d")]
struct Params {
    num_particles: usize,
    photon_flux: PhotonFlux,
    number_density: NumberDensity,
    source_pos: Length,
}

impl Params {
    fn cell_size(&self) -> Length {
        Length::megaparsec(BOX_SIZE_MEGAPARSEC) / self.num_particles as f64
    }
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
    let sweep_params = sim
        .add_parameter_type_and_get_result::<SweepParameters>()
        .clone();
    add_box_size(&mut sim, &params);
    add_grid(&mut sim, &params, &sweep_params);
    add_source(&mut sim, &params);
    if **sim.get_resource::<WorldSize>().unwrap() > 1 {
        panic!("1d test not supported on multiple cores - to fix this, initialize the sweep cells after domain decomposition")
    }
    sim.write_output(true)
        .add_plugin(SourcePlugin)
        .add_parameters_explicitly(Cosmology::NonCosmological)
        .add_startup_system_to_stage(
            StartupStages::InsertDerivedComponents,
            initialize_sweep_components_system,
        )
        .add_plugin(SweepPlugin);
    sim
}

fn add_source(sim: &mut Simulation, params: &Params) {
    let area = params.cell_size().squared();
    if sim.on_main_rank() {
        sim.insert_resource(Sources {
            sources: vec![Source {
                position: VecLength::new(params.source_pos, Length::zero(), Length::zero()),
                rate: params.photon_flux * area,
            }],
        });
    } else {
        sim.insert_resource(Sources { sources: vec![] });
    }
}

fn add_box_size(sim: &mut Simulation, params: &Params) {
    let cell_size = params.cell_size();
    let min = VecLength::zero();
    let max = VecLength::new(
        Length::megaparsec(BOX_SIZE_MEGAPARSEC),
        cell_size,
        cell_size,
    );
    let extent = Extent {
        min,
        max,
        center: (min + max) / 2.0,
    };
    let box_size = SimulationBox::new(extent);
    sim.insert_resource(box_size);
}

fn add_grid(sim: &mut Simulation, params: &Params, sweep_params: &SweepParameters) {
    let cell_size = params.cell_size();
    let grid_setup = {
        let sweep_params = sweep_params.clone();
        move |commands: Commands,
              box_size: Res<SimulationBox>,
              world_size: Res<WorldSize>,
              world_rank: Res<WorldRank>| {
            init_cartesian_grid_system(
                commands,
                box_size,
                NumCellsSpec::CellSize(cell_size),
                world_size,
                world_rank,
                sweep_params.periodic,
            )
        }
    };
    sim.add_startup_system(grid_setup);
}

fn initialize_sweep_components_system(
    mut commands: Commands,
    local_particles: Query<Entity, With<LocalParticle>>,
    params: Res<Params>,
) {
    for entity in local_particles.iter() {
        let mass_density = params.number_density * PROTON_MASS;
        commands.entity(entity).insert((
            Density(mass_density),
            components::IonizedHydrogenFraction(Dimensionless::dimensionless(1e-10)),
            components::Temperature(Temperature::kelvins(1000.0)),
            components::PhotonRate(PhotonRate::zero()),
            components::Source(PhotonRate::zero()),
        ));
    }
}
