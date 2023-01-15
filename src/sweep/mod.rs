pub mod components;
mod count_by_dir;
mod direction;
mod parameters;
mod site;
mod task;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
pub use parameters::SweepParameters;

use self::components::AbsorptionRate;
use self::components::Flux;
use self::components::HydrogenIonizationFraction;
use self::components::Source;
use self::count_by_dir::CountByDir;
use self::direction::Directions;
use self::site::Site;
use self::task::Task;
use crate::components::Density;
use crate::components::Position;
use crate::components::Timestep;
use crate::grid::Cell;
use crate::grid::FaceArea;
use crate::grid::Neighbour;
use crate::grid::RemoteNeighbour;
use crate::prelude::*;
use crate::simulation::RaxiomPlugin;
use crate::units::Dimensionless;
use crate::units::PhotonFlux;
use crate::units::SourceRate;
use crate::units::PROTON_MASS;

type PriorityQueue<T> = std::collections::binary_heap::BinaryHeap<T>;

type CellQuery<'w, 's> = Particles<'w, 's, (Entity, &'static Cell, &'static Position)>;
type SiteQuery<'w, 's> = Particles<
    'w,
    's,
    (
        &'static mut Site,
        &'static mut AbsorptionRate,
        &'static mut components::Flux,
        &'static Density,
        &'static HydrogenIonizationFraction,
    ),
>;
type SourceQuery<'w, 's> = Particles<'w, 's, &'static mut Source>;

#[derive(Named)]
pub struct SweepPlugin;

impl RaxiomPlugin for SweepPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(
            SimulationStartupStages::InsertComponents,
            initialize_directions_system,
        )
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertDerivedComponents,
            initialize_sites_system,
        )
        .add_required_component::<HydrogenIonizationFraction>()
        .add_required_component::<Source>()
        .add_derived_component::<components::Flux>()
        .add_derived_component::<AbsorptionRate>()
        .add_system(init_counts_system.before(sweep_system))
        .add_system(reset_sites_system.before(sweep_system))
        .add_system(sweep_system)
        .add_system(ionize_hydrogen_system.after(sweep_system))
        .add_parameter_type::<SweepParameters>();
    }
}

struct Sweep<'w, 's> {
    directions: Directions,
    cells: CellQuery<'w, 's>,
    sites: SiteQuery<'w, 's>,
    sources: SourceQuery<'w, 's>,
    to_solve: PriorityQueue<Task>,
    remaining_to_solve_count: CountByDir,
}

impl<'w, 's> Sweep<'w, 's> {
    fn run(
        directions: &Directions,
        cells: CellQuery<'w, 's>,
        sites: SiteQuery,
        sources: SourceQuery,
    ) {
        let remaining_to_solve = CountByDir::new(directions.len(), cells.iter().count());
        let mut solver = Sweep {
            cells,
            sites,
            sources,
            to_solve: PriorityQueue::new(),
            directions: directions.clone(),
            remaining_to_solve_count: remaining_to_solve,
        };
        solver.add_initial_tasks();
        solver.solve();
    }

    fn add_initial_tasks(&mut self) {
        let tasks = self
            .directions
            .enumerate()
            .flat_map(|(dir_index, dir)| {
                self.cells
                    .iter()
                    .filter(|entry| {
                        let cell1 = entry.1;
                        // Importantly, the !face_points_upwind cannot
                        // be changed to face_points_downwind, because
                        // we need to be inclusive of all faces, even
                        // those that have zero dot product with the
                        // face normal.
                        cell1
                            .neighbours
                            .iter()
                            .all(|(face, neighbour)| !face.points_upwind(dir) || neighbour.is_boundary())
                    })
                    .map(move |(entity, _, _)| Task {
                        entity,
                        dir: dir_index,
                        flux: PhotonFlux::zero(),
                    })
            })
            .collect();
        self.to_solve = tasks;
    }

    fn solve(&mut self) {
        let remaining_to_send = 0;
        while self.remaining_to_solve_count.total() > 0 || remaining_to_send > 0 {
            if self.to_solve.is_empty() {
                self.receive_messages();
            }
            while let Some(task) = self.to_solve.pop() {
                self.solve_task(task);
            }
            self.send_all_messages();
        }
    }

    fn receive_messages(&self) {}

    fn send_all_messages(&self) {}

    fn solve_eq(&mut self, task: &Task) -> PhotonFlux {
        let cell = self.cells.get_component::<Cell>(task.entity).unwrap();
        let density = **self.sites.get_component::<Density>(task.entity).unwrap();
        let ionized_hydrogen_abundance = **self
            .sites
            .get_component::<HydrogenIonizationFraction>(task.entity)
            .unwrap();
        let hydrogen_number_density = density / PROTON_MASS * (1.0 - ionized_hydrogen_abundance);
        let source = match self.sources.get_component::<Source>(task.entity) {
            Ok(source) => **source / self.directions.len() as f64,
            Err(_) => SourceRate::zero(),
        };
        let sigma = crate::units::SWEEP_HYDROGEN_ONLY_CROSS_SECTION;
        let mut absorption_rate = self
            .sites
            .get_component_mut::<AbsorptionRate>(task.entity)
            .unwrap();
        let flux = task.flux + source;
        let absorbed_fraction = 1.0 - (-hydrogen_number_density * sigma * cell.size).exp();
        **absorption_rate += flux * absorbed_fraction;
        let mut cell_flux = self.sites.get_component_mut::<Flux>(task.entity).unwrap();
        **cell_flux += flux;
        flux * (1.0 - absorbed_fraction)
    }

    fn solve_task(&mut self, task: Task) {
        let outgoing_flux = self.solve_eq(&task);
        let cell = self.cells.get_component::<Cell>(task.entity).unwrap();
        self.remaining_to_solve_count.reduce(task.dir);
        // This is very inefficient, let's see if this ever becomes a bottleneck
        let neighbours = cell.neighbours.clone();
        let total_effective_area: FaceArea = cell
            .iter_downwind_faces(&self.directions[task.dir])
            .map(|face| face.area * face.normal.dot(*self.directions[task.dir]))
            .sum();
        for (face, neighbour) in neighbours.iter() {
            if face.points_downwind(&self.directions[task.dir]) {
                let effective_area = face.area * face.normal.dot(*self.directions[task.dir]);
                let flux_this_cell = outgoing_flux * (effective_area / total_effective_area);
                match neighbour {
                    Neighbour::Local(neighbour_entity) => {
                        self.handle_local_neighbour(flux_this_cell, &task, *neighbour_entity)
                    }
                    Neighbour::Remote(remote) => self.handle_remote_neighbour(remote),
                    Neighbour::Boundary => {}
                }
            }
        }
    }

    fn handle_local_neighbour(
        &mut self,
        incoming_flux: PhotonFlux,
        task: &Task,
        neighbour: Entity,
    ) {
        let mut site = self.sites.get_component_mut::<Site>(neighbour).unwrap();
        let num_remaining = site.num_missing_upwind.reduce(task.dir);
        site.flux[*task.dir] += incoming_flux;
        if num_remaining == 0 {
            self.to_solve.push(Task {
                dir: task.dir,
                entity: neighbour,
                flux: site.flux[*task.dir],
            })
        }
    }

    fn handle_remote_neighbour(&mut self, _remote: &RemoteNeighbour) {
        todo!()
    }
}

fn init_counts_system(
    cells: CellQuery,
    mut sites: SiteQuery,
    parameters: Res<SweepParameters>,
    directions: Res<Directions>,
) {
    for (entity, cell, _) in cells.iter() {
        let mut site = sites.get_component_mut::<Site>(entity).unwrap();
        site.num_missing_upwind = CountByDir::new(parameters.directions.len(), 0);
        for (dir_index, dir) in directions.enumerate() {
            for (face, neighbour) in cell.neighbours.iter() {
                if !neighbour.is_boundary() && face.points_upwind(dir) {
                    site.num_missing_upwind[dir_index] += 1;
                }
            }
        }
    }
}

fn sweep_system(
    directions: Res<Directions>,
    cells: CellQuery,
    sites: SiteQuery,
    sources: SourceQuery,
) {
    Sweep::run(&directions, cells, sites, sources);
}

fn ionize_hydrogen_system(
    mut particles: Particles<(&mut HydrogenIonizationFraction, &AbsorptionRate, &Timestep)>,
) {
    for (mut ionized_fraction, absorption_rate, timestep) in particles.iter_mut() {
        **ionized_fraction += **absorption_rate * **timestep;
        **ionized_fraction = ionized_fraction.clamp(
            Dimensionless::dimensionless(0.0),
            Dimensionless::dimensionless(1.0),
        );
    }
}

fn initialize_sites_system(mut commands: Commands, cells: CellQuery, directions: Res<Directions>) {
    for (entity, _, _) in cells.iter() {
        commands.entity(entity).insert((
            Site {
                num_missing_upwind: CountByDir::empty(),
                flux: (0..directions.len()).map(|_| PhotonFlux::zero()).collect(),
            },
            AbsorptionRate(PhotonFlux::zero()),
            components::Flux(PhotonFlux::zero()),
        ));
    }
}

fn initialize_directions_system(mut commands: Commands, parameters: Res<SweepParameters>) {
    let directions: Directions = (&parameters.directions).into();
    commands.insert_resource(directions);
}

fn reset_sites_system(
    mut sites: Particles<(&mut Site, &mut AbsorptionRate, &mut components::Flux)>,
    directions: Res<Directions>,
) {
    for (mut site, mut rate, mut flux) in sites.iter_mut() {
        for (i, _) in directions.enumerate() {
            site.flux[*i] = PhotonFlux::zero();
        }
        **rate = PhotonFlux::zero();
        **flux = PhotonFlux::zero();
    }
}
