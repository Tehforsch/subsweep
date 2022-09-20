use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use bevy::prelude::debug;

use crate::named::Named;
use crate::simulation::Simulation;
use crate::simulation::TenetPlugin;

pub struct ReadParametersError(String);

pub trait Parameters
where
    Self: Sized,
{
    fn from_empty() -> Result<Self, ReadParametersError>;
}

impl<T> Parameters for T
where
    T: Default,
{
    fn from_empty() -> Result<Self, ReadParametersError> {
        Ok(<T as Default>::default())
    }
}

struct ParameterFileContents(String);

impl Simulation {
    pub fn add_parameters_from_file(&mut self, parameter_file_name: &Path) -> &mut Self {
        let contents = fs::read_to_string(parameter_file_name).unwrap_or_else(|_| {
            panic!(
                "Failed to read parameter file at {:?}",
                &parameter_file_name
            )
        });
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

impl<T> TenetPlugin for ParameterPlugin<T>
where
    T: Named + Parameters + Sync + Send + 'static + for<'de> serde::Deserialize<'de>,
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
        let parameter_file_contents = &sim.get_resource::<ParameterFileContents>().unwrap_or_else(|| panic!("No parameter file contents resource available while reading parameters for {} - failed to call add_parameter_file_contents?", T::name())).0;
        let parameters = Self::get_parameter_struct_from_parameter_file_contents(
            T::name(),
            parameter_file_contents,
        );
        sim.insert_resource(parameters);
    }
}

impl<T: Parameters + Sync + Send + 'static + for<'de> serde::Deserialize<'de>> ParameterPlugin<T> {
    fn get_parameter_struct_from_parameter_file_contents(
        name: &str,
        parameter_file_contents: &str,
    ) -> T {
        let all_parameters: serde_yaml::Value =
            serde_yaml::from_str(parameter_file_contents).unwrap();
        all_parameters
            .get(name)
            .map(|plugin_parameters| {
                serde_yaml::from_value(plugin_parameters.clone())
                    .expect("Failed to read parameter file")
            })
            .unwrap_or_else(|| match T::from_empty() {
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

    use crate::named::Named;
    use crate::parameters::ParameterFileContents;
    use crate::parameters::ParameterPlugin;
    use crate::parameters::Parameters;
    use crate::parameters::ReadParametersError;
    use crate::simulation::Simulation;

    #[derive(Deserialize, Default, Named)]
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
        let mut sim = Simulation::new();
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
        sim.add_plugin(ParameterPlugin::<Parameters1>::default())
            .add_plugin(ParameterPlugin::<Parameters2>::default());
        let params1 = sim.unwrap_resource::<Parameters1>();
        let params2 = sim.unwrap_resource::<Parameters2>();
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
        impl Parameters for Parameters1 {
            fn from_empty() -> Result<Self, crate::parameters::ReadParametersError> {
                Err(ReadParametersError("Missing required param 'i'".into()))
            }
        }
        let mut sim = Simulation::new();
        sim.insert_resource(ParameterFileContents("".into()));
        sim.add_plugin(ParameterPlugin::<Parameters1>::default());
    }
}
