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

use super::get_parameters;
use super::run_once;
use super::tenet_plugin::TenetPlugin;
use super::AlreadyAddedLabels;
use crate::communication::WorldRank;
use crate::named::Named;

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
        run_once::<T>(self, |sim| {
            plugin.build_once_everywhere(sim);
            if !sim.has_world_rank() {
            } else if get_parameters::<WorldRank>(sim).is_main() {
                plugin.build_once_on_main_rank(sim);
            } else {
                plugin.build_once_on_other_ranks(sim);
            }
        });
        plugin.build_everywhere(self);
        if !self.has_world_rank() {
        } else if get_parameters::<WorldRank>(self).is_main() {
            plugin.build_on_main_rank(self);
        } else {
            plugin.build_on_other_ranks(self);
        }
        self
    }

    fn has_world_rank(&self) -> bool {
        if !self.contains_resource::<WorldRank>() {
            warn!("World rank not present during plugin initialization, this should only happen in tests");
            false
        } else {
            true
        }
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
        self.0.run()
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
    pub fn panic_if_already_added<P: Named>(&mut self) {
        let mut labels = self.get_resource_or_insert_with(AlreadyAddedLabels::default);
        if !labels.0.insert(P::name()) {
            panic!("Added twice: {}", P::name())
        }
    }
}
