use super::Simulation;
use crate::named::Named;

pub trait TenetPlugin: Named {
    /// A conditional determines whether the plugin should be built at
    /// all. Defaults to true. Note that build_always_once is run before
    /// should_build and will always run, regardless of the result of
    /// should_build
    fn should_build(&self, _sim: &Simulation) -> bool {
        true
    }

    /// Determines whether a panic should occur if this plugin is
    /// added twice. This defaults to true and can be turned of
    /// for example in generic plugins which have the same name
    /// but will be added (potentially) multiple times.
    fn allow_adding_twice(&self) -> bool {
        false
    }

    /// Called once per plugin type, regardless of the value of
    /// allow_adding_twice. Can be useful to set up anything
    /// required by the should_build condition.
    fn build_always_once(&self, _sim: &mut Simulation) {}

    /// Called on every rank on every initialization of the plugin
    fn build_everywhere(&self, _sim: &mut Simulation) {}

    /// Called on the main rank on every initialization of the plugin
    fn build_on_main_rank(&self, _sim: &mut Simulation) {}

    /// Called on all ranks except the main rank on every initialization of the plugin
    fn build_on_other_ranks(&self, _sim: &mut Simulation) {}

    /// Called on every rank once per plugin type.
    /// Only relevant for generic plugins.
    fn build_once_everywhere(&self, _sim: &mut Simulation) {}

    /// Called on the main rank once per plugin type.
    /// Only relevant for generic plugins.
    fn build_once_on_main_rank(&self, _sim: &mut Simulation) {}

    /// Called on all ranks except the main rank once per plugin type.
    /// Only relevant for generic plugins.
    fn build_once_on_other_ranks(&self, _sim: &mut Simulation) {}
}
