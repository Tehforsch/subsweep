mod raxiom_plugin;

use std::collections::HashSet;

use bevy::app::PluginGroupBuilder;
use bevy::ecs::event::Event;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::system::Resource;
use bevy::prelude::debug;
use bevy::prelude::warn;
use bevy::prelude::App;
use bevy::prelude::Component;
use bevy::prelude::Mut;
use bevy::prelude::Plugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::Stage;
use bevy::prelude::StageLabel;
use bevy::prelude::World;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
pub use raxiom_plugin::RaxiomPlugin;

use crate::communication::WorldRank;
use crate::domain::ExchangeDataPlugin;
use crate::io::input::ComponentInput;
use crate::io::input::DatasetInputPlugin;
use crate::io::output::OutputPlugin;
use crate::io::to_dataset::ToDataset;
use crate::named::Named;
use crate::parameter_plugin::ParameterFileContents;
use crate::parameter_plugin::ParameterPlugin;
use crate::parameter_plugin::Parameters;

#[derive(Default)]
pub struct Simulation {
    pub app: App,
    labels: HashSet<&'static str>,
    parameter_sections: HashSet<String>,
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
        self.run_without_finalize();
        Simulation::finalize();
    }

    pub fn finalize() {
        #[cfg(feature = "mpi")]
        crate::communication::MPI_UNIVERSE.drop();
    }

    /// Runs the simulation without calling MPI_FINALIZE.  This should
    /// only be used for benchmarks and other use cases where multiple
    /// simulations are run.  Make sure to call finalize() explicitly
    /// after the last run
    pub fn run_without_finalize(&mut self) {
        // Since this is called from tests which don't have a BaseCommunication plugin, make sure we only unwrap
        // world rank if it exists and default to validating otherwise.
        if self.get_resource::<WorldRank>().is_none() || self.on_main_rank() {
            self.validate();
        }
        self.app.run();
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
        T: Parameters,
    {
        self.parameter_sections.insert(T::name().into());
        self.add_plugin(ParameterPlugin::<T>::default());
        self
    }

    pub fn add_parameter_type_and_get_result<T>(&mut self) -> &T
    where
        T: Parameters,
    {
        self.add_parameter_type::<T>();
        self.unwrap_resource::<T>()
    }

    pub fn add_parameters_explicitly<T: Parameters>(&mut self, parameters: T) -> &mut Self {
        self.insert_resource(parameters);
        self
    }

    pub fn add_component<T>(&mut self, input: ComponentInput) -> &mut Self
    where
        T: Equivalence + ToDataset + Component,
        <T as Equivalence>::Out: MatchesRaw,
    {
        self.add_plugin(ExchangeDataPlugin::<T>::default())
            .add_plugin(OutputPlugin::<T>::default());
        match input {
            ComponentInput::Required => {
                self.add_plugin(DatasetInputPlugin::<T>::default());
            }
            ComponentInput::Derived => {}
        }
        self
    }

    pub fn add_required_component<T>(&mut self) -> &mut Self
    where
        T: Equivalence + ToDataset + Component,
        <T as Equivalence>::Out: MatchesRaw,
    {
        self.add_component::<T>(ComponentInput::Required)
    }

    pub fn add_derived_component<T>(&mut self) -> &mut Self
    where
        T: Equivalence + ToDataset + Component,
        <T as Equivalence>::Out: MatchesRaw,
    {
        self.add_component::<T>(ComponentInput::Derived)
    }

    fn validate(&self) {
        let contents = self.unwrap_resource::<ParameterFileContents>();
        let mut unused = vec![];
        for param in contents.get_section_names() {
            if !self.parameter_sections.contains(&param) {
                unused.push(param);
            }
        }
        if !unused.is_empty() {
            panic!(
                "Unused parameter sections: {}. Used parameter sections: {}",
                unused.join(", "),
                self.parameter_sections
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::named::Named;
    use crate::parameter_plugin::ParameterFileContents;
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

    #[test]
    #[should_panic(expected = "Unused parameter sections")]
    fn panic_on_unused_parameter_section() {
        let mut sim = Simulation::default();
        let contents = "
parameters1:
  x:
    3.0
";
        sim.insert_resource(ParameterFileContents(contents.into()));
        sim.run();
    }
}
