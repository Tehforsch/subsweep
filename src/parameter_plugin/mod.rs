use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use bevy::prelude::debug;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use serde::Deserialize;
use serde_yaml::Mapping;
use serde_yaml::Value;

use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;

pub trait Parameters: Named + for<'de> Deserialize<'de> + Sync + Send + 'static {}

impl<T> Parameters for T where T: Named + for<'de> Deserialize<'de> + Sync + Send + 'static {}

pub struct ReadParametersError(String);

#[derive(Deref, DerefMut)]
pub(super) struct ParameterFileContents(pub String);

impl ParameterFileContents {
    pub fn get_section_names(&self) -> Vec<String> {
        self.value()
            .as_mapping()
            .unwrap_or(&Mapping::default())
            .keys()
            .map(|key| {
                key.as_str()
                    .expect("Non-string parameter section")
                    .to_owned()
            })
            .collect()
    }

    fn value(&self) -> Value {
        serde_yaml::from_str::<Value>(&self.0).unwrap_or(Value::Null)
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
        self.insert_resource(ParameterFileContents(contents));
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

fn from_empty<T>() -> Result<T, ReadParametersError>
where
    T: Parameters,
{
    serde_yaml::from_str::<T>("").map_err(|_| {
        ReadParametersError(format!(
            "No section {} in parameter file. This section cannot be left out",
            T::name()
        ))
    })
}

impl<T> RaxiomPlugin for ParameterPlugin<T>
where
    T: Parameters,
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
            debug!("Parameters for {} already present", T::name());
            false
        } else {
            true
        }
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        let parameter_file_contents = &sim.get_resource::<ParameterFileContents>().unwrap_or_else(|| panic!("No parameter file contents resource available while reading parameters for {} - failed to call add_parameters_from_file?", T::name()));
        let parameters = Self::get_parameter_struct_from_parameter_file_contents(
            T::name(),
            parameter_file_contents,
        );
        sim.insert_resource(parameters);
    }
}

impl<T: Parameters> ParameterPlugin<T> {
    fn get_parameter_struct_from_parameter_file_contents(
        name: &str,
        parameter_file_contents: &ParameterFileContents,
    ) -> T {
        parameter_file_contents
            .value()
            .get(name)
            .map(|plugin_parameters| {
                // The following is a workaround for deserializing a serde_yaml::Value,
                // which fails when visiting dimensionless quantities (which will be interpreted as floats)
                serde_yaml::from_str(&serde_yaml::to_string(plugin_parameters).unwrap())
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to read parameter file section \"{}\": \n{}",
                            T::name(),
                            err
                        )
                    })
            })
            .unwrap_or_else(|| match from_empty() {
                Ok(params) => {
                    debug!(
                        "Parameter section missing for '{}', assuming defaults",
                        name
                    );
                    params
                }
                Err(msg) => {
                    panic!("Failed to read parameters: {}", &msg.0)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::ParameterFileContents;
    use super::ParameterPlugin;
    use crate::named::Named;
    use crate::simulation::Simulation;

    #[derive(Clone, Deserialize, Default, Named)]
    #[name = "parameters1"]
    struct Parameters1 {
        i: i32,
    }

    #[derive(Deserialize, Default, Named)]
    #[name = "parameters2"]
    struct Parameters2 {
        s: String,
        #[serde(default)]
        d: String,
    }

    #[test]
    fn parameter_plugin() {
        let mut sim = Simulation::default();
        sim.insert_resource(ParameterFileContents(
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
        #[derive(Deserialize, Named)]
        #[name = "parameters1"]
        struct Parameters1 {
            _i: i32,
        }

        let mut sim = Simulation::default();
        sim.insert_resource(ParameterFileContents("".into()));
        sim.add_plugin(ParameterPlugin::<Parameters1>::default());
    }

    #[test]
    fn allow_leaving_out_struct_with_complete_set_of_defaults() {
        #[derive(Deserialize, Named)]
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
        sim.insert_resource(ParameterFileContents("".into()));
        let params = sim.add_parameter_type_and_get_result::<Parameters1>();
        assert_eq!(params.i, 15);
        assert_eq!(params.x, 12.0);
    }

    #[test]
    fn allow_defaults_from_type_default() {
        #[derive(Deserialize, Named)]
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
        sim.insert_resource(ParameterFileContents(contents.into()));
        let params = sim.add_parameter_type_and_get_result::<Parameters1>();
        assert_eq!(params.x, 2.0);
        assert_eq!(params.i, 0);
    }
}
