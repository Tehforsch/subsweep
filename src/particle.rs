use bevy_ecs::archetype::Archetypes;
use bevy_ecs::component::Components;
use bevy_ecs::prelude::Bundle;
use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::Query;
use bevy_ecs::prelude::With;
use log::debug;
use mpi::traits::Equivalence;

use crate::communication::Rank;
use crate::components::Position;
use crate::named::Named;
use crate::prelude::Simulation;
use crate::prelude::StartupStages;
use crate::simulation::RaxiomPlugin;

#[derive(
    Component, Clone, Debug, PartialEq, Eq, Hash, Equivalence, Copy, Named, PartialOrd, Ord,
)]
#[name = "id"]
pub struct ParticleId {
    pub index: u32,
    pub rank: Rank,
}

impl ParticleId {
    pub fn test(index: usize) -> Self {
        Self {
            index: index as u32,
            rank: 0,
        }
    }
}

#[derive(Component)]
pub struct LocalParticle;

#[derive(Component)]
pub struct HaloParticle {
    pub rank: Rank,
}

/// A convenience type to query for particles.
/// ```
/// # use raxiom::components::Position;
/// # use raxiom::components::Mass;
/// # use raxiom::prelude::Particles;
/// fn my_system(particles: Particles<(&Position, &Mass)>) {
///     for (pos, mass) in particles.iter() {
///        println!("Particle with mass  {} at {} m", mass.in_kilograms(), pos.in_meters());
///     }
/// }
/// ```
pub type Particles<'world, 'state, T, F = ()> = Query<'world, 'state, T, (With<LocalParticle>, F)>;

/// A convenience type to query for halo particles.
pub type HaloParticles<'world, 'state, T, F = ()> =
    Query<'world, 'state, T, (With<HaloParticle>, F)>;

#[derive(Bundle)]
pub struct LocalParticleBundle {
    pos: Position,
    _local: LocalParticle,
}

#[derive(Named)]
pub struct ParticlePlugin;

impl RaxiomPlugin for ParticlePlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(StartupStages::Final, count_types_system);
    }
}

fn count_types_system(archetypes: &Archetypes, components: &Components) {
    // Count archetypes
    // Filter out empty archetype and resource archetype
    let relevant_archetypes: Vec<_> = archetypes
        .iter()
        .filter(|archetype| !archetype.is_empty())
        .collect();
    debug!("Num archetypes on main rank: {}", relevant_archetypes.len());
    for archetype in relevant_archetypes.iter() {
        debug!("----");
        for component in archetype.components() {
            let info = components.get_info(component).unwrap();
            debug!("  {}", info.name());
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::Component;
    use bevy_ecs::prelude::With;
    use bevy_ecs::prelude::World;

    use crate::prelude::LocalParticle;
    use crate::prelude::Particles;
    use crate::test_utils::run_system_on_world;

    #[test]
    fn particles_query_respects_filters() {
        #[derive(Component)]
        struct A;
        #[derive(Component)]
        struct B;
        let mut world = World::default();
        world.spawn((A, B, LocalParticle));
        world.spawn((A, B, LocalParticle));
        world.spawn((A, LocalParticle));
        world.spawn((A,));
        fn system(particles: Particles<&A, With<B>>) {
            assert_eq!(particles.iter().count(), 2);
        }
        run_system_on_world(&mut world, system);
    }

    #[test]
    fn particles_query_respects_tuple_filters() {
        #[derive(Component)]
        struct A;
        #[derive(Component)]
        struct B;
        #[derive(Component)]
        struct C;
        let mut world = World::default();
        world.spawn((A, B, C, LocalParticle));
        world.spawn((A, B, LocalParticle));
        world.spawn((A, LocalParticle));
        world.spawn((A,));
        fn system(particles: Particles<&A, (With<B>, With<C>)>) {
            assert_eq!(particles.iter().count(), 1);
        }
        run_system_on_world(&mut world, system);
    }
}
