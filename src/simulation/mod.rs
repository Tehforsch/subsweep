mod raxiom_plugin;

use std::collections::HashMap;
use std::collections::HashSet;

use bevy::ecs::event::Event;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::schedule::StateData;
use bevy::ecs::schedule::SystemDescriptor;
use bevy::ecs::schedule::SystemLabelId;
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
use bevy::prelude::SystemSet;
use bevy::prelude::World;
use derive_traits::RaxiomParameters;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;
pub use raxiom_plugin::RaxiomPlugin;

use crate::communication::WorldRank;
use crate::domain::ExchangeDataPlugin;
use crate::io::input::ComponentInput;
use crate::io::input::DatasetInputPlugin;
use crate::io::output::OutputPlugin;
use crate::io::to_dataset::ToDataset;
use crate::io::DatasetDescriptor;
use crate::io::DatasetShape;
use crate::io::InputDatasetDescriptor;
use crate::memory::ComponentMemoryUsagePlugin;
use crate::named::Named;
use crate::parameter_plugin::ParameterFileContents;
use crate::parameter_plugin::ParameterPlugin;

#[derive(Default)]
pub struct Simulation {
    pub app: App,
    labels: HashSet<&'static str>,
    parameter_sections: HashSet<String>,
    ordering_labels: HashMap<&'static str, Vec<SystemLabelId>>,
    current_communication_tag: i32,
    pub read_initial_conditions: bool,
    pub write_output: bool,
}

impl Simulation {
    #[cfg(test)]
    pub fn test() -> Self {
        use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;

        let mut sim = Self::default();
        sim.insert_resource(ReportExecutionOrderAmbiguities);
        sim
    }

    pub fn get_next_tag(&mut self) -> i32 {
        self.current_communication_tag += 1;
        self.current_communication_tag
    }

    pub fn read_initial_conditions(&mut self, read_initial_conditions: bool) -> &mut Self {
        self.read_initial_conditions = read_initial_conditions;
        self
    }

    pub fn write_output(&mut self, write_output: bool) -> &mut Self {
        self.write_output = write_output;
        self
    }

    pub fn already_added<P: Named>(&mut self) -> bool {
        !self.labels.insert(P::name())
    }

    pub fn add_plugin<T: RaxiomPlugin>(&mut self, plugin: T) -> &mut Self {
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

    pub fn add_startup_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.app.add_startup_stage_after(target, label, stage);
        self
    }

    pub fn add_state(&mut self, s: impl StateData) -> &mut Self {
        self.app.add_state(s);
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

    pub fn add_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.app.add_system_to_stage(stage_label, system);
        self
    }

    pub fn add_system_set_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system_set: SystemSet,
    ) -> &mut Self {
        self.app.add_system_set_to_stage(stage_label, system_set);
        self
    }

    /// Adds a system to a stage and makes sure that this system and
    /// all other systems with the same Marker type are executed by
    /// the scheduler in the exact order that they were added in, with
    /// the first system to be added being executed first.
    pub fn add_well_ordered_system_to_stage<Params, Marker: Named>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
        label: SystemLabelId,
    ) -> &mut Self {
        let marker = Marker::name();
        if !self.ordering_labels.contains_key(marker) {
            self.ordering_labels.insert(marker, vec![]);
        }
        let labels = self.ordering_labels.get_mut(marker).unwrap();
        // The following is a bit overly complicated because I am
        // confused about ParallelSystemDescriptors - how do I get one
        // without calling .after() or .label() or .before()?
        if labels.is_empty() {
            self.app.add_system_to_stage(stage_label, system);
            labels.push(label);
            self
        } else {
            let mut system: SystemDescriptor = system.after(labels[0]);
            for label in labels.iter() {
                system = system.after(*label);
            }
            labels.push(label);
            self.app.add_system_to_stage(stage_label, system);
            self
        }
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
        if !self.has_world_rank()
            || self.on_main_rank() && self.contains_resource::<ParameterFileContents>()
        {
            self.validate();
        }
        self.app.run();
    }

    pub fn update(&mut self) {
        self.app.update()
    }

    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.app.world.get_resource::<T>()
    }

    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<Mut<T>> {
        self.app.world.get_resource_mut::<T>()
    }

    pub fn unwrap_resource<T: Resource>(&self) -> &T {
        self.app.world.get_resource::<T>().unwrap()
    }

    pub fn unwrap_resource_mut<T: Resource>(&mut self) -> Mut<T> {
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

    pub fn contains_resource<T: Resource>(&self) -> bool {
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
        T: RaxiomParameters,
    {
        self.parameter_sections
            .insert(T::unwrap_section_name().into());
        self.add_plugin(ParameterPlugin::<T>::default());
        self
    }

    pub fn add_parameter_type_and_get_result<T>(&mut self) -> &T
    where
        T: RaxiomParameters,
    {
        self.add_parameter_type::<T>();
        self.unwrap_resource::<T>()
    }

    pub fn add_parameters_explicitly<T: RaxiomParameters>(&mut self, parameters: T) -> &mut Self {
        self.insert_resource(parameters);
        self
    }

    pub fn get_parameters<T: RaxiomParameters>(&self) -> &T {
        self.get_resource::<T>().unwrap()
    }

    pub fn add_component<T>(
        &mut self,
        input: ComponentInput<T>,
        output_descriptor: DatasetDescriptor,
    ) -> &mut Self
    where
        T: Equivalence + Named + ToDataset + Component,
        <T as Equivalence>::Out: MatchesRaw,
    {
        self.add_component_no_io::<T>();
        self.add_plugin(OutputPlugin::<T>::from_descriptor(output_descriptor));
        match input {
            ComponentInput::Required(descriptor) => {
                self.add_plugin(DatasetInputPlugin::<T>::from_descriptor(descriptor));
            }
            ComponentInput::Derived => {}
        }
        self
    }

    pub fn add_required_component<T>(&mut self) -> &mut Self
    where
        T: Equivalence + ToDataset + Component + Named,
        <T as Equivalence>::Out: MatchesRaw,
    {
        self.add_component::<T>(
            ComponentInput::Required(InputDatasetDescriptor::new(
                DatasetDescriptor::default_for::<T>(),
                DatasetShape::OneDimensional,
            )),
            DatasetDescriptor::default_for::<T>(),
        )
    }

    pub fn add_derived_component<T>(&mut self) -> &mut Self
    where
        T: Equivalence + ToDataset + Component + Named,
        <T as Equivalence>::Out: MatchesRaw,
    {
        self.add_component::<T>(
            ComponentInput::Derived,
            DatasetDescriptor::default_for::<T>(),
        )
    }

    pub fn add_component_no_io<T>(&mut self) -> &mut Self
    where
        T: Clone + Named + Equivalence + Component,
        <T as Equivalence>::Out: MatchesRaw,
    {
        if self.has_world_rank() {
            self.add_plugin(ExchangeDataPlugin::<T>::default());
        }
        self.add_plugin(ComponentMemoryUsagePlugin::<T>::default());
        self
    }

    fn validate(&self) {
        let contents = self.unwrap_resource::<ParameterFileContents>();
        let mut unused = vec![];
        for param in contents.get_section_names() {
            if !self.parameter_sections.contains(param) {
                unused.push(param.to_owned());
            }
        }
        // This will eventually be replaced by a allow_unused method on
        // the parameters trait, but that will probably require a proper
        // derive macro for the parameters trait.
        if unused.len() == 1 && unused[0] == "visualization" {
            return;
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
        sim.add_parameter_file_contents(contents.into());
        sim.run();
    }
}
