use bevy::ecs::query::QueryItem;
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::SystemParam;
use bevy::prelude::Res;
use bevy::utils::HashSet;

use super::time_bins::TimeBins;
use super::ActiveTimestep;
use crate::prelude::Particles;

#[derive(SystemParam)]
pub struct ActiveParticles<'w, 's, T, Q, F = ()>
where
    Q: WorldQuery + 'static,
    F: WorldQuery + 'static,
    T: Sync + Send + 'static,
{
    query: Particles<'w, 's, Q, F>,
    timebins: Res<'w, TimeBins<T>>,
    active_timestep: Res<'w, ActiveTimestep>,
}

impl<'w, 's, T, Q, F> ActiveParticles<'w, 's, T, Q, F>
where
    Q: WorldQuery + 'static,
    F: WorldQuery + 'static,
    T: Sync + Send + 'static,
{
    pub fn iter(&'w self) -> impl Iterator<Item = ROQueryItem<Q>> + 'w
    where
        Q: WorldQuery,
        F: WorldQuery,
    {
        self.timebins[**self.active_timestep]
            .iter()
            .map(move |x| self.query.get(*x).unwrap())
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = QueryItem<'a, Q>>
    where
        Q: WorldQuery,
        F: WorldQuery,
    {
        // Some notes on this pretty bad implementation:
        // From the outside, this is actually really simple, I want to write something like
        //
        // for entity in timebins[**active_timestep].iter() {
        //     let mut item = query.get_mut(*entity).unwrap();
        // }
        //
        // As demonstrated by iter(), this doesn't have to be that
        // hard. However, writing an iterator for the mutable version
        // of this was impossible to me.  I am not proud of this
        // solution at all. It allocates unnecessarily and is probably
        // a lot slower than a proper version of this would be.
        // approximately 2 hours trying to get the lifetimes right by
        // writing a custom iterator struct to do this but didn't
        // manage.  If this ever becomes a bottleneck, I will revisit
        // this.
        //
        // SAFETY: We check for entity uniqueness here, so that
        // get_unchecked is safe. This is something that could
        // possibly be removed if we ensure that the entities in the
        // timebins are always unique
        //
        let entities: Vec<_> = self.timebins[**self.active_timestep].iter().collect();
        let num = entities.len();
        let entities = entities.into_iter().collect::<HashSet<_>>();
        assert_eq!(num, entities.len());
        let pointers: Vec<_> = entities
            .into_iter()
            .map(|x| unsafe { self.query.get_unchecked(*x) }.unwrap())
            .collect();
        pointers.into_iter()
    }
}
