mod tenet_plugin;

use std::collections::HashSet;

use bevy::ecs::event::Event;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::system::Resource;
use bevy::prelude::warn;
use bevy::prelude::App;
use bevy::prelude::Mut;
use bevy::prelude::Plugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::Stage;
use bevy::prelude::StageLabel;
use bevy::prelude::World;
use serde::Deserialize;
pub use tenet_plugin::TenetPlugin;

use crate::communication::WorldRank;
use crate::named::Named;
use crate::parameters::ParameterPlugin;
use crate::parameters::Parameters;

#[derive(Default)]
struct RunOnceLabels(HashSet<&'static str>);

#[derive(Default)]
struct AlreadyAddedLabels(HashSet<&'static str>);
#[derive(Default)]

pub struct Simulation(pub App);

impl Simulation {
    pub fn new() -> Self {
        Self(App::new())
    }

    pub fn add_plugin<T: Sync + Send + 'static + TenetPlugin>(&mut self, plugin: T) -> &mut Self {
        if !plugin.should_build(self) {
            return self;
        }
        if !plugin.allow_adding_twice() {
            self.panic_if_already_added::<T>()
        }
        self.run_once::<T>(|sim| {
            plugin.build_once_everywhere(sim);
            if !sim.has_world_rank() {
            } else if sim.on_main_rank() {
                plugin.build_once_on_main_rank(sim);
            } else {
                plugin.build_once_on_other_ranks(sim);
            }
        });
        plugin.build_everywhere(self);
        if !self.has_world_rank() {
        } else if self.on_main_rank() {
            plugin.build_on_main_rank(self);
        } else {
            plugin.build_on_other_ranks(self);
        }
        self
    }

    pub fn maybe_add_plugin<T: Sync + Send + 'static + TenetPlugin>(
        &mut self,
        plugin: Option<T>,
    ) -> &mut Self {
        if let Some(plugin) = plugin {
            self.add_plugin(plugin);
        }
        self
    }

    pub fn add_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.0.add_stage_after(target, label, stage);
        self
    }

    pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.0.add_system(system);
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.0.insert_resource(resource);
        self
    }

    pub fn insert_non_send_resource<R: 'static>(&mut self, resource: R) -> &mut Self {
        self.0.insert_non_send_resource(resource);
        self
    }

    pub fn add_bevy_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        self.0.add_plugin(plugin);
        self
    }

    pub fn add_bevy_plugins<T: PluginGroup>(&mut self, group: T) -> &mut Self {
        self.0.add_plugins(group);
        self
    }

    pub fn add_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.0.add_system_to_stage(stage_label, system);
        self
    }

    pub fn add_startup_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.0.add_startup_system_to_stage(stage_label, system);
        self
    }

    pub fn add_startup_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.0.add_startup_system(system);
        self
    }

    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: Event,
    {
        self.0.add_event::<T>();
        self
    }

    pub fn run(&mut self) {
        self.0.run();
        #[cfg(feature = "mpi")]
        crate::communication::MPI_UNIVERSE.drop();
    }

    pub fn update(&mut self) {
        self.0.update()
    }

    pub fn get_resource<T: Sync + Send + 'static>(&self) -> Option<&T> {
        self.0.world.get_resource::<T>()
    }

    pub fn get_resource_mut<T: Sync + Send + 'static>(&mut self) -> Option<Mut<T>> {
        self.0.world.get_resource_mut::<T>()
    }

    pub fn unwrap_resource<T: Sync + Send + 'static>(&self) -> &T {
        self.0.world.get_resource::<T>().unwrap()
    }

    pub fn unwrap_resource_mut<T: Sync + Send + 'static>(&mut self) -> Mut<T> {
        self.0.world.get_resource_mut::<T>().unwrap()
    }

    pub fn unwrap_non_send_resource_mut<R: 'static>(&mut self) -> Mut<'_, R> {
        self.0.world.non_send_resource_mut::<R>()
    }

    pub fn get_resource_or_insert_with<R: Resource>(
        &mut self,
        func: impl FnOnce() -> R,
    ) -> Mut<'_, R> {
        self.0.world.get_resource_or_insert_with(func)
    }

    pub fn contains_resource<T: Sync + Send + 'static>(&self) -> bool {
        self.get_resource::<T>().is_some()
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.0.world
    }

    /// Panics if a named item was (accidentally) added twice
    fn panic_if_already_added<P: Named>(&mut self) {
        let mut labels = self.get_resource_or_insert_with(AlreadyAddedLabels::default);
        if !labels.0.insert(P::name()) {
            panic!("Added twice: {}", P::name())
        }
    }

    fn has_world_rank(&self) -> bool {
        if !self.contains_resource::<WorldRank>() {
            warn!("World rank not present during plugin initialization, this should only happen in tests");
            false
        } else {
            true
        }
    }

    pub fn on_main_rank(&self) -> bool {
        self.unwrap_resource::<WorldRank>().is_main()
    }

    pub fn run_once<P: Named>(&mut self, f: impl Fn(&mut Simulation)) {
        let mut labels = self.get_resource_or_insert_with(RunOnceLabels::default);
        if labels.0.insert(P::name()) {
            f(self);
        }
    }

    pub fn add_parameter_type<T>(&mut self, name: &str) -> &mut Self
    where
        T: Sync + Send + 'static + Clone + Parameters + for<'de> Deserialize<'de>,
    {
        self.add_plugin(ParameterPlugin::<T>::new(name));
        self
    }

    pub fn add_parameter_type_and_get_result<T>(&mut self, name: &str) -> T
    where
        T: Sync + Send + 'static + Clone + Parameters + for<'de> Deserialize<'de>,
    {
        self.add_plugin(ParameterPlugin::<T>::new(name));
        self.unwrap_resource::<T>().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::named::Named;
    use crate::simulation::Simulation;
    use crate::simulation::TenetPlugin;

    #[test]
    #[should_panic]
    fn add_plugin_twice() {
        #[derive(Named)]
        #[name = "my_plugin"]
        struct MyPlugin;
        impl TenetPlugin for MyPlugin {}
        let mut sim = Simulation::new();
        sim.add_plugin(MyPlugin);
        sim.add_plugin(MyPlugin);
    }
}
