use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use bevy::prelude::App;
use bevy::prelude::Plugin;

pub fn add_parameter_file_contents(app: &mut App, parameter_file_name: &Path) {
    let contents = fs::read_to_string(&parameter_file_name).expect(&format!(
        "Failed to read parameter file at {:?}",
        &parameter_file_name
    ));
    app.world
        .insert_resource(ParameterFileContents(contents.clone()));
}

struct ParameterFileContents(String);

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

impl<T: Sync + Send + 'static + for<'de> serde::Deserialize<'de>> Plugin for ParameterPlugin<T> {
    fn build(&self, app: &mut App) {
        let name = self.name.clone();
        let parameter_file_contents = &app.world.get_resource::<ParameterFileContents>().expect("No parameter file contents resource available - failed to call add_parameter_file_contents?").0;
        let parameters =
            Self::get_parameter_struct_from_parameter_file_contents(&name, parameter_file_contents);
        app.world.insert_resource(parameters);
    }
}

impl<T: Sync + Send + 'static + for<'de> serde::Deserialize<'de>> ParameterPlugin<T> {
    fn get_parameter_struct_from_parameter_file_contents(
        name: &str,
        parameter_file_contents: &str,
    ) -> T {
        let all_parameters: serde_yaml::Value =
            serde_yaml::from_str(parameter_file_contents).unwrap();
        let plugin_parameters = all_parameters
            .get(name)
            .expect(&format!("Parameter section missing for '{}'", name));
        serde_yaml::from_value(plugin_parameters.clone()).expect("Failed to read parameter file")
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::Deserialize;

    use crate::parameters::ParameterFileContents;
    use crate::parameters::ParameterPlugin;

    #[test]
    fn parameter_plugin() {
        #[derive(Deserialize)]
        struct Parameters1 {
            i: i32,
        }

        #[derive(Deserialize)]
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
}
