# Raxiom
Raxiom is an (experimental) astrophysical simulation library
using the [`bevy`](https://docs.rs/bevy/*/bevy) entity component system.

## Goals
### Testability
In order to properly understand the multi-physical nature of
astrophysics, simulations require accurate treatment of many different
phenomena, such as gravity, hydrodynamics, chemistry, the formation of
stars and black holes, radiation transport and many more.

At the same time, correctness of the results is of fundamental
importance to all scientific software. This is a challenging problem
for software in general and becomes even more difficult in the context
of astrophysical simulations where even an integration test strategy
which compares the results to theory is often impossible for anything
other than the simplest edge cases, due to the lack of available analytical
predictions. This is often true even for individual components of the
simulation (consider, for example, gravity in which the solution to
the three body problem can already not be expressed in closed
form). The usual approach to deal with this problem is to compare the
results of the simulations of a particular system with those of other
simulation codes solving the same system. This increases the
confidence in the results and helps in understanding the different
properties of the employed numerical methods.

While comparison to numerical solutions (and analytical predictions
when possible) is absolutely essential to numerical astrophysics, this
project is an attempt at building a simulation library that
additionally lends itself to being tested in the software development
way - via unit and integration tests. Many conventional simulation
codes suffer from their complexity. Often having to manage different
particle types (gas, dark matter, sink particles, ...) each of which
interacts in a variety of ways with the other types and itself while
still keeping the code performant, simple, understandable and testable
is difficult.

This is the reason why Raxiom uses an entity component system (ECS) as
the underlying strategy. ECS have the advantage of expressing shared
properties in a really simple way. For example, consider the fact that
both gas and dark matter particles have a position and a velocity and
should be integrated in time. In conventional simulation codes, this
often results in code duplication or complicated mechanisms. In an ECS
framework, this is solved very elegantly, by simply adding a position
and velocity component for both particle types and adding an time
integration system which runs on every entity that has both position
and velocity as components. Testing such functionality in conventional
codes is made very difficult by the fact that position and velocity
will often be entries in giant data structures containing often
hundreds of other quantities which will have to be created before any
of the code runs properly. In an ECS framework, testing this system is
very simple, because the test simply needs to ensure that some entity
with `Position and velocity is present in order to check the
functionality of the system. This is a critical difference and ensures
that testing scales to a large codebase without having to be
refactored continuously. For example raxiom contains an "integration
test" that compares the results of the parallelized gravity force
tree-walk (with opening angle 0) to the direct, O(n^2) sum. In raxioms
framework, this test is easy to write and maintain and its existence
allows experimenting with different strategies for domain
decomposition and gravity calculation without having to worry about
accidentally introducing a bug.

In Bevy's terminology, the different particle types of the simulation
correspond to archetypes of the ECS and simply consist of entities
with different sets of components. One noteworthy advantage of the ECS
approach is that it makes adding functionality to a subset of the
particles not only easy, but also possible without any memory
overhead, by adding an empty marker struct as a component, which will
change the underlying archetype without requiring (a significant
amount of) additional memory. In a simplified example, one could add
an empty `Tracer` component to gas particles and then add a system
which writes the position of every gas particle with the `Tracer`
component to an output file.

### Modularity
In addition to being easy to test, an ECS framework also has the
advantage of not having a defined data structure. This means that the
decisions about which subsystems and components to add to the
simulation can be done at runtime instead of compile time without
incurring any memory overhead, which is often impossible in
conventional simulations.

Additionally, Raxiom enables building simulations consisting of a
number of different phenomena in the "Bevy way", by treating these
phenomena as modular plugins which can be added and configured
individually.  The reason Raxiom exposes a library structure instead
of consisting of a single, all-powerful, configurable binary is that
this allows the user to easily add additional behavior to the
simulation without requiring an ever increasing set of configuration
flags and parameters for the binary. Injecting such custom behavior
into the "main loop" of the code is made easy by the structure of ECS
in general and Bevys amazing modularity in particular.

Here is how this might look in practice:

```rust no_run
use raxiom::prelude::*;
use raxiom::components::*;
use bevy::prelude::*;

fn main() {
    let mut sim = SimulationBuilder::new();
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

### Unit safety
One of the goals of Raxiom is to deal in physical quantities instead
of unspecified floats. This is important for three reasons:
1. It is incredibly easy to introduce mistakes when dealing with units
   as a human, possibly causing incorrect results.
2. Constantly having to deal with units uses up mental capacity of
   researchers that would be much better employed doing something
   useful. This applies to both writing simulation code as well as
   running simulations.
3. As a bonus, having quantities in the code makes the code more
   readable. Consider:
    ```rust ignore
    fn foo(x: f64, y: f64) -> f64 {
        x / y
    }
    
    fn bar(x: Length, y: Time) -> Velocity {
        x / y
    }
    ```
    Although both methods are terribly named, `bar` is readable, while it is entirely unclear what `foo` does and which argument goes where.

This is why Raxiom is unit-safe from the beginning to the end.

1. Simulation parameters are written and read with explicit units. For example, here is how an example parameter file looks:
    ```yaml
    simulation:
      minimum_timestep: 1 kyr
      final_time: 1 Myr
    output:
      time_between_snapshots: 10 kyr
    hydrodynamics:
      min_smoothing_length: 0.01 pc
    ```
    These parameters are parsed and then internally translated to code units. For example, the `simulation` section of this parameter file is given by
    ```rust ignore
    #[derive(Clone, Deserialize, Named)]
    #[name = "simulation"]
    pub struct SimulationParameters {
        pub minimum_timestep: Time,
        pub final_time: Option<Time>,
    }
    ```
2. Initial conditions are read with explicit units. For example, the positions of the particle might be read from a hdf5 dataset containing attributes about how the position is translated into a fixed unit system, such as SI or CGS. This ensures that even when the underlying code units are changed, initial conditions will still be read in the exact same way (resulting in the same physical quantities).

3. Output is written with units, analogous to how initial conditions are read.

4. The code is written entirely in compile-time units. This means that any unit error in any of the equations in the code
    ```rust compile_fail
    # use raxiom::units::{Time, Length, Velocity};
    let time = Time::seconds(1.0);
    let length = Length::meters(10.0);
    let velocity: Velocity = length * time;
    ```
    results in a compile time error:
    ```text
    55 |         let velocity: Velocity = length * time;
       |                                  ^^^^^^^^^^^^^ expected `dimension::Dimension { length: 1, time: -1, mass: 0, temperature: 0 }`, found `dimension::Dimension { length: 1, time: 1, mass: 0, temperature: 0 }`
       |
       = note: expected constant `dimension::Dimension { length: 1, time: -1, mass: 0, temperature: 0 }`
                  found constant `dimension::Dimension { length: 1, time: 1, mass: 0, temperature: 0 }`
    ```
    This is a zero-cost abstraction, meaning that the unit checks happen at compile time and the physical quantities are simply represented by a float at runtime.
    I wrote my own [`diman`](https://github.com/tehforsch/diman) crate for this purpose. There are other great compile time units libraries in rust, but I am not aware of any which use the (unstable) const generics which makes for comparatively nice error messages. Moreover, having a custom library specifically built for this purpose will make it easy to include things such as comoving units or different coordinate systems whenever needed.

5. As the final step, the creatively named [`pyxiom`](https://github.com/tehforsch/pyxiom) contains python bindings to read Raxiom's output files and translates any datasets that the user wants to read directly into [`astropy`](https://github.com/astropy/astropy) units.

With all these steps, Raxiom ensures that both user and programmer never have to interact with units themselves.


### Performance and parallelism
In order to allow highly parallel simulations, Raxiom uses
[`rsmpi`](https://github.com/rsmpi/rsmpi) for MPI communication
between multiple Bevy apps. Moreover, each Bevy app is parallelized
internally using Bevys inherent parallelism and parallel queries.
This reduces the amount of MPI ranks needed and might even allow
automatic "zero-effort" latency hiding by running non-conflicting
systems in parallel - systems which are currently blocked by
communication latency or delay do not block program execution.

In order to keep simulations testable, Raxiom can run on two different
types of communication. Real simulations run on MPI, for performance.
However, tests can be run using a local communication strategy in
which each bevy app is run in a different thread and the threads
communicate via channels.  This not only makes testing easier but also
enables running tests on more ranks than there are cores available,
which is impossible in MPI.

In order to make sure that all of the code can be run with both
communication strategies and to keep the code simple, Raxiom's systems
communicate by requesting a corresponding `Communicator`. For example,
consider this system which determines the extent of the simulation
(i.e. the bounding box of all of the particles in the simulations).

```rust ignore
fn determine_global_extent_system(
    particles: Particles<&Position>,
    mut extent_communicator: Communicator<CommunicatedOption<Extent>>,
    mut global_extent: ResMut<GlobalExtent>,
) {
    let extent = Extent::from_positions(particles.iter().map(|x| &x.0));
    let all_extents = (*extent_communicator).all_gather(&extent.into());
    let all_extents: Vec<Extent> = all_extents.into_iter().filter_map(|x| x.into()).collect();
    *global_extent = GlobalExtent(
        Extent::get_all_encompassing(all_extents.iter())
            .expect("Failed to find simulation extent - are there no particles?")
            .pad(),
    );
}
```

The `Communicator` struct can perform the usual MPI routines such as
`Send`, `Recv`, `Allgather`, `Allreduce`, ... More complex
communication strategies are implemented in special communicators,
such as `ExchangeCommunicator` (which handles the exchange of
particles across domain boundaries). All of these communicators
usually employ MPI, but switch to local communication if the
compile-time feature `mpi` is not present.

## Features
At this point, Raxiom supports a Barnes-Hut tree gravity solver as
well as smoothed particle hydrodynamics in 2D and 3D. It reads and
writes Hdf5 files for initial conditions and output.  It allows easily
writing custom plugins with their own parameters. For debugging and
fun, small simulations can be visualized live with the
`VisualizationPlugin`. See the `examples` directory for more information.
