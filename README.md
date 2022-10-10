# Raxiom
Raxiom is an (experimental) astrophysical simulation library
using the rust game engine [`bevy`](https://docs.rs/bevy/*/bevy).

At this point, Raxiom supports a Barnes-Hut tree gravity solver as
well as smoothed particle hydrodynamics. In order to allow highly
parallel simulations, Raxiom uses MPI communication between
multiple Bevy apps. Moreover, each Bevy app is parallelized
internally using Bevys inherent parallelism and parallel queries.
This reduces the amount of MPI ranks needed and might even allow
automatic "zero-effort" latency hiding by running non-conflicting
systems in parallel - systems which are currently blocked by
communication latency or delay do not block program execution.

In order to properly understand the multi-physical nature of
astrophysics, simulations require accurate treatment of many
different phenomena, such as gravity, hydrodynamics, chemistry,
the formation of stars and black holes, radiation transport and
many more.  Raxiom enables building such simulations in the "Bevy
way", treating these phenomena as modular plugins which can be
added and configured individually.  The reason Raxiom exposes a
library structure instead of consisting of a single, all-powerful,
configurable binary is that this allows the user to easily add
additional behavior to the simulation without requiring an ever
increasing set of configuration flags and parameters for the
binary. Injecting such custom behavior into the "main loop"
of the code is made easy by the structure of ECS in general and
Bevys amazing modularity in particular.

Here is how this might look in practice:

## A basic example
```rust
use raxiom::prelude::*;
use bevy::prelude::*;

fn main() {
    let mut sim = SimulationBuilder::mpi();
    sim.parameters_from_relative_path(file!(), "parameters.yml")
        .update_from_command_line_options()
        .build()
        .add_plugin(GravityPlugin)
        .add_plugin(HydrodynamicsPlugin)
        .add_system(my_custom_behavior)
        .run();
}

fn my_custom_behavior(
    mut commands: Commands,
    mut particles: Particles<(Entity, &Position, &Mass)>,
) {
    for (entity, position, mass) in particles.iter() {
        if **mass > units::Mass::kilograms(9000.0) {
            // The mass is too high, we should refine this into two particles
            commands.entity(entity).despawn();
            commands.spawn_bundle((position.clone(), Mass(**mass / 2.0)));
            commands.spawn_bundle((position.clone(), Mass(**mass / 2.0)));
        }
    }
}
```
