pub mod parameter_file_contents;

use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use bevy::prelude::debug;
pub use derive_custom::RaxiomParameters;
use serde::Deserialize;
use serde::Serialize;

use self::parameter_file_contents::Override;
pub use self::parameter_file_contents::ParameterFileContents;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

pub trait RaxiomParameters: Serialize + for<'de> Deserialize<'de> + Sync + Send + 'static {
    fn section_name() -> Option<&'static str>;

    fn unwrap_section_name() -> &'static str {
        Self::section_name()
            .unwrap_or_else(|| panic!("Called unwrap_section_name on unnamed parameter struct."))
    }
}

impl<T> RaxiomParameters for T
where
    T: Named + Serialize + for<'de> Deserialize<'de> + Sync + Send + 'static,
{
    fn section_name() -> Option<&'static str> {
        Some(<T as Named>::name())
    }
}

impl Simulation {
    pub fn add_parameters_from_file(&mut self, parameter_file_name: &Path) -> &mut Self {
        let contents = fs::read_to_string(parameter_file_name).unwrap_or_else(|_| {
            panic!(
                "Failed to read parameter file at {:?}",
                &parameter_file_name
            )
        });
        self.add_parameter_file_contents(contents)
    }

    pub fn add_parameter_file_contents(&mut self, contents: String) -> &mut Self {
        self.insert_resource(ParameterFileContents::new(contents));
        self
    }

    pub fn with_parameter_overrides(&mut self, overrides: Vec<Override>) -> &mut Self {
        self.get_resource_mut::<ParameterFileContents>()
            .unwrap()
            .with_overrides(overrides);
        self
    }
}

#[derive(Named)]
pub struct ParameterPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for ParameterPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T> RaxiomPlugin for ParameterPlugin<T>
where
    T: RaxiomParameters,
{
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        // In tests, we want to be able to insert the parameters
        // directly into the sim, without having to read a parameter
        // file which is why we only add the plugin if the parameter
        // struct isn't already present
        if sim.contains_resource::<T>() {
            debug!("Parameters for {:?} already present", T::section_name());
            false
        } else {
            true
        }
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        let mut parameter_file_contents = sim.get_resource_mut::<ParameterFileContents>().unwrap_or_else(|| panic!("No parameter file contents resource available while reading parameters for {:?} - failed to call add_parameters_from_file?", T::section_name()));
        let parameters: T = parameter_file_contents.extract_parameter_struct();
        sim.insert_resource(parameters);
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde::Serialize;

    use super::ParameterFileContents;
    use super::ParameterPlugin;
    use crate::named::Named;
    use crate::simulation::Simulation;

    #[derive(Clone, Serialize, Deserialize, Default, Named)]
    #[name = "parameters1"]
    struct Parameters1 {
        i: i32,
    }

    #[derive(Serialize, Deserialize, Default, Named)]
    #[name = "parameters2"]
    struct Parameters2 {
        s: String,
        #[serde(default)]
        d: String,
    }

    #[test]
    fn parameter_plugin() {
        let mut sim = Simulation::default();
        sim.insert_resource(ParameterFileContents::new(
            "
parameters1:
  i:
    1
parameters2:
  s:
   'hi'"
                .into(),
        ));
        let params1 = sim
            .add_parameter_type_and_get_result::<Parameters1>()
            .clone();
        let params2 = sim.add_parameter_type_and_get_result::<Parameters2>();
        assert_eq!(params1.i, 1);
        assert_eq!(params2.s, "hi");
        assert_eq!(params2.d, "");
    }

    #[test]
    #[should_panic]
    fn do_not_accept_missing_required_parameter_section() {
        #[derive(Serialize, Deserialize, Named)]
        #[name = "parameters1"]
        struct Parameters1 {
            _i: i32,
        }

        let mut sim = Simulation::default();
        sim.add_parameter_file_contents("".into());
        sim.add_plugin(ParameterPlugin::<Parameters1>::default());
    }

    #[test]
    fn allow_leaving_out_struct_with_complete_set_of_defaults() {
        #[derive(Serialize, Deserialize, Named)]
        #[name = "parameters1"]
        struct Parameters1 {
            #[serde(default = "get_default_i")]
            i: i32,
            #[serde(default = "get_default_x")]
            x: f32,
        }

        fn get_default_i() -> i32 {
            15
        }

        fn get_default_x() -> f32 {
            12.0
        }
        let mut sim = Simulation::default();
        sim.add_parameter_file_contents("".into());
        let params = sim.add_parameter_type_and_get_result::<Parameters1>();
        assert_eq!(params.i, 15);
        assert_eq!(params.x, 12.0);
    }

    #[test]
    fn allow_defaults_from_type_default() {
        #[derive(Serialize, Deserialize, Named)]
        #[name = "parameters1"]
        struct Parameters1 {
            #[serde(default)]
            i: i32,
            x: f32,
        }

        let mut sim = Simulation::default();
        let contents = "
parameters1:
  x:
    2.0";
        sim.add_parameter_file_contents(contents.into());
        let params = sim.add_parameter_type_and_get_result::<Parameters1>();
        assert_eq!(params.x, 2.0);
        assert_eq!(params.i, 0);
    }
}
