use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::system::Resource;
use bevy::prelude::App;
use bevy::prelude::Mut;
use bevy::prelude::Plugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::Stage;
use bevy::prelude::StageLabel;
use bevy::prelude::World;

use super::tenet_plugin::IntoPlugin;
use super::tenet_plugin::TenetPlugin;

#[derive(Default)]
pub struct Simulation(pub App);

impl Simulation {
    pub fn new() -> Self {
        Self(App::new())
    }

    pub fn add_tenet_plugin<T: Sync + Send + 'static + TenetPlugin>(
        &mut self,
        plugin: T,
    ) -> &mut Self {
        self.0.add_plugin(IntoPlugin::from(plugin));
        self
    }

    pub fn add_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        self.0.add_stage(label, stage);
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

    pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        self.0.add_plugin(plugin);
        self
    }

    pub fn add_plugins<T: PluginGroup>(&mut self, group: T) -> &mut Self {
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

    pub fn add_startup_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.0.add_startup_system(system);
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

    pub fn unwrap_resource<T: Sync + Send + 'static>(&self) -> &T {
        self.0.world.get_resource::<T>().unwrap()
    }

    pub fn unwrap_resource_mut<T: Sync + Send + 'static>(&mut self) -> Mut<T> {
        self.0.world.get_resource_mut::<T>().unwrap()
    }

    pub fn unwrap_non_send_resource_mut<R: 'static>(&mut self) -> Mut<'_, R> {
        self.0.world.non_send_resource_mut::<R>()
    }

    pub fn world(&mut self) -> &mut World {
        &mut self.0.world
    }
}
