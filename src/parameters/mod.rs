use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use bevy::prelude::debug;
use bevy::prelude::App;
use bevy::prelude::Plugin;

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

pub fn add_parameter_file_contents(app: &mut App, parameter_file_name: &Path) {
    let contents = fs::read_to_string(parameter_file_name).unwrap_or_else(|_| {
        panic!(
            "Failed to read parameter file at {:?}",
            &parameter_file_name
        )
    });
    app.world.insert_resource(ParameterFileContents(contents));
}

pub struct ParameterPlugin<T> {
    _marker: PhantomData<T>,
    name: String,
}

impl<T> ParameterPlugin<T> {
    pub fn new(name: &str) -> Self {
        Self {
            _marker: PhantomData::default(),
            name: name.into(),
        }
    }
}

impl<T: Parameters + Sync + Send + 'static + for<'de> serde::Deserialize<'de>> Plugin
    for ParameterPlugin<T>
{
    fn build(&self, app: &mut App) {
        let name = self.name.clone();
        // In tests, we want to be able to insert the parameters
        // directly into the app, without having to read a parameter file
        // which is why we check here whether the parameter struct is already present
        if app.world.get_resource::<T>().is_some() {
            debug!("Parameters for {} already present", &name);
            return;
        }
        let parameter_file_contents = &app.world.get_resource::<ParameterFileContents>().unwrap_or_else(|| panic!("No parameter file contents resource available while reading parameters for {} - failed to call add_parameter_file_contents?", &name)).0;
        let parameters =
            Self::get_parameter_struct_from_parameter_file_contents(&name, parameter_file_contents);
        app.world.insert_resource(parameters);
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
    use bevy::prelude::*;
    use serde::Deserialize;

    use crate::parameters::ParameterFileContents;
    use crate::parameters::ParameterPlugin;
    use crate::parameters::Parameters;
    use crate::parameters::ReadParametersError;

    #[test]
    fn parameter_plugin() {
        #[derive(Deserialize, Default)]
        struct Parameters1 {
            i: i32,
        }

        #[derive(Deserialize, Default)]
        struct Parameters2 {
            s: String,
            #[serde(default)]
            d: String,
        }

        let mut app = App::new();
        app.insert_resource(ParameterFileContents(
            "
parameters1:
  i:
    1
parameters2:
  s:
   'hi'"
                .into(),
        ));
        app.add_plugin(ParameterPlugin::<Parameters1>::new("parameters1"))
            .add_plugin(ParameterPlugin::<Parameters2>::new("parameters2"));
        let params1 = app.world.get_resource::<Parameters1>().unwrap();
        let params2 = app.world.get_resource::<Parameters2>().unwrap();
        assert_eq!(params1.i, 1);
        assert_eq!(params2.s, "hi");
        assert_eq!(params2.d, "");
    }

    #[test]
    #[should_panic]
    fn do_not_accept_missing_required_parameter_section() {
        #[derive(Deserialize)]
        struct Parameters1 {
            _i: i32,
        }

        impl Parameters for Parameters1 {
            fn from_empty() -> Result<Self, crate::parameters::ReadParametersError> {
                Err(ReadParametersError("Missing required param 'i'".into()))
            }
        }
        let mut app = App::new();
        app.insert_resource(ParameterFileContents("".into()));
        app.add_plugin(ParameterPlugin::<Parameters1>::new("parameters1"));
    }
}
