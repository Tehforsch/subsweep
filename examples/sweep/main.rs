#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::collections::HashMap;

use bevy::prelude::*;
use raxiom::components::Position;
use raxiom::grid::Cell;
use raxiom::grid::Neighbour;
use raxiom::grid::NeighbourKind;
use raxiom::prelude::*;
use raxiom::units::Length;
use raxiom::units::VecLength;

#[raxiom_parameters("sweep_postprocess")]
struct Parameters {
    cell_size: Length,
}

fn main() {
    let mut sim = SimulationBuilder::new();
    sim.parameters_from_relative_path(file!(), "parameters.yml")
        .headless(false)
        .write_output(false)
        .read_initial_conditions(false)
        .update_from_command_line_options()
        .build()
        .add_startup_system(init_grid_system)
        .add_plugin(SweepPlugin)
        .add_parameter_type::<Parameters>()
        .run();
}

fn init_grid_system(
    mut commands: Commands,
    box_size: Res<SimulationBox>,
    parameters: Res<Parameters>,
) {
    let num_cells_per_dimension_float = (box_size.side_lengths() / parameters.cell_size).value();
    let num_cells_per_dimension_x = num_cells_per_dimension_float.x.floor() as i64;
    let num_cells_per_dimension_y = num_cells_per_dimension_float.y.floor() as i64;
    let mut map = HashMap::new();
    for x in 0..num_cells_per_dimension_x {
        for y in 0..num_cells_per_dimension_y {
            let entity = commands
                .spawn((
                    LocalParticle,
                    Position(VecLength::new(
                        x as f64 * parameters.cell_size,
                        y as f64 * parameters.cell_size,
                    )),
                ))
                .id();
            map.insert((x, y), entity);
        }
    }
    for x in 0..num_cells_per_dimension_x {
        for y in 0..num_cells_per_dimension_y {
            let entity = map[&(x, y)];
            let neighbours = [
                (x - 1, y - 1),
                (x + 1, y - 1),
                (x - 1, y + 1),
                (x + 1, y + 1),
            ]
            .iter()
            .filter_map(|(x_neigh, y_neigh)| {
                if (0..num_cells_per_dimension_x).contains(x_neigh)
                    && (0..num_cells_per_dimension_y).contains(y_neigh)
                {
                    Some(Neighbour {
                        entity: map[&(*x_neigh, *y_neigh)],
                        kind: NeighbourKind::Local,
                    })
                } else {
                    None
                }
            })
            .collect();
            commands.entity(entity).insert(Cell { neighbours });
        }
    }
}
