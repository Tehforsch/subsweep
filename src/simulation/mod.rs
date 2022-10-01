mod raxiom_plugin;

use std::collections::HashSet;

use bevy::app::PluginGroupBuilder;
use bevy::ecs::event::Event;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::system::Resource;
use bevy::prelude::debug;
use bevy::prelude::warn;
use bevy::prelude::App;
use bevy::prelude::Mut;
use bevy::prelude::Plugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::Stage;
use bevy::prelude::StageLabel;
use bevy::prelude::World;
pub use raxiom_plugin::RaxiomPlugin;
use serde::Deserialize;

use crate::communication::WorldRank;
use crate::named::Named;
use crate::parameter_plugin::ParameterPlugin;

#[derive(Default)]
pub struct Simulation {
    pub app: App,
    labels: HashSet<&'static str>,
}

impl Simulation {
    pub fn already_added<P: Named>(&mut self) -> bool {
        !self.labels.insert(P::name())
    }

    pub fn add_plugin<T: Sync + Send + 'static + RaxiomPlugin>(&mut self, plugin: T) -> &mut Self {
        let already_added = self.already_added::<T>();
        if !already_added {
            plugin.build_always_once(self);
        }
        if !plugin.should_build(self) {
            debug!("Skip plugin: {}", T::name());
            return self;
        }
        debug!(" Add plugin: {}", T::name());
        if !plugin.allow_adding_twice() && already_added {
            panic!("Added twice: {}", T::name())
        }
        if !already_added {
            plugin.build_once_everywhere(self);
            if !self.has_world_rank() {
            } else if self.on_main_rank() {
                plugin.build_once_on_main_rank(self);
            } else {
                plugin.build_once_on_other_ranks(self);
            }
        }
        plugin.build_everywhere(self);
        if !self.has_world_rank() {
        } else if self.on_main_rank() {
            plugin.build_on_main_rank(self);
        } else {
            plugin.build_on_other_ranks(self);
        }
        self
    }

    pub fn maybe_add_plugin<T: Sync + Send + 'static + RaxiomPlugin>(
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
        self.app.add_stage_after(target, label, stage);
        self
    }

    pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.app.add_system(system);
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.app.insert_resource(resource);
        self
    }

    pub fn insert_non_send_resource<R: 'static>(&mut self, resource: R) -> &mut Self {
        self.app.insert_non_send_resource(resource);
        self
    }

    pub fn add_bevy_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        self.app.add_plugin(plugin);
        self
    }

    pub fn add_bevy_plugins<T: PluginGroup>(&mut self, group: T) -> &mut Self {
        self.app.add_plugins(group);
        self
    }

    pub fn add_bevy_plugins_with<T, F>(&mut self, group: T, func: F) -> &mut Self
    where
        T: PluginGroup,
        F: FnOnce(&mut PluginGroupBuilder) -> &mut PluginGroupBuilder,
    {
        self.app.add_plugins_with(group, func);
        self
    }

    pub fn add_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.app.add_system_to_stage(stage_label, system);
        self
    }

    pub fn add_startup_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.app.add_startup_system_to_stage(stage_label, system);
        self
    }

    pub fn add_startup_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.app.add_startup_system(system);
        self
    }

    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: Event,
    {
        self.app.add_event::<T>();
        self
    }

    pub fn run(&mut self) {
        self.app.run();
        #[cfg(feature = "mpi")]
        crate::communication::MPI_UNIVERSE.drop();
    }

    pub fn update(&mut self) {
        self.app.update()
    }

    pub fn get_resource<T: Sync + Send + 'static>(&self) -> Option<&T> {
        self.app.world.get_resource::<T>()
    }

    pub fn get_resource_mut<T: Sync + Send + 'static>(&mut self) -> Option<Mut<T>> {
        self.app.world.get_resource_mut::<T>()
    }

    pub fn unwrap_resource<T: Sync + Send + 'static>(&self) -> &T {
        self.app.world.get_resource::<T>().unwrap()
    }

    pub fn unwrap_resource_mut<T: Sync + Send + 'static>(&mut self) -> Mut<T> {
        self.app.world.get_resource_mut::<T>().unwrap()
    }

    pub fn unwrap_non_send_resource_mut<R: 'static>(&mut self) -> Mut<'_, R> {
        self.app.world.non_send_resource_mut::<R>()
    }

    pub fn get_resource_or_insert_with<R: Resource>(
        &mut self,
        func: impl FnOnce() -> R,
    ) -> Mut<'_, R> {
        self.app.world.get_resource_or_insert_with(func)
    }

    pub fn contains_resource<T: Sync + Send + 'static>(&self) -> bool {
        self.get_resource::<T>().is_some()
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.app.world
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

    pub fn add_parameter_type<T>(&mut self) -> &mut Self
    where
        T: Named + Sync + Send + 'static + for<'de> Deserialize<'de>,
    {
        self.add_plugin(ParameterPlugin::<T>::default());
        self
    }

    pub fn add_parameter_type_and_get_result<T>(&mut self) -> &T
    where
        T: Named + Sync + Send + 'static + for<'de> Deserialize<'de>,
    {
        self.add_plugin(ParameterPlugin::<T>::default());
        self.unwrap_resource::<T>()
    }

    pub fn add_parameters_explicitly<T: Sync + Send + 'static>(
        &mut self,
        parameters: T,
    ) -> &mut Self {
        self.insert_resource(parameters);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::named::Named;
    use crate::simulation::RaxiomPlugin;
    use crate::simulation::Simulation;

    #[test]
    #[should_panic]
    fn add_plugin_twice() {
        #[derive(Named)]
        #[name = "my_plugin"]
        struct MyPlugin;
        impl RaxiomPlugin for MyPlugin {}
        let mut sim = Simulation::default();
        sim.add_plugin(MyPlugin);
        sim.add_plugin(MyPlugin);
    }
}
