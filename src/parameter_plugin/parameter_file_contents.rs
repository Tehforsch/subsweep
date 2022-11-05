use bevy::prelude::debug;
use serde_yaml::Mapping;
use serde_yaml::Value;

use super::Parameters;
use super::ReadParametersError;

#[derive(Debug, Clone)]
pub struct Override {
    pub keys: Vec<String>,
    pub value: Value,
}

pub struct ParameterFileContents {
    contents: String,
    overrides: Vec<Override>,
}

fn get_sub_value_by_keys<'a>(value: &'a mut Value, keys: &[String]) -> &'a mut Value {
    if keys.len() == 0 {
        value
    } else {
        get_sub_value_by_keys(
            value
                .as_mapping_mut()
                .unwrap()
                .get_mut(&keys[0])
                .unwrap_or_else(|| {
                    panic!(
                        "Override key `{}` not found (remaining keys: {:?})",
                        &keys[0], &keys
                    )
                }),
            &keys[1..],
        )
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

impl ParameterFileContents {
    pub fn new(contents: String) -> Self {
        Self {
            contents,
            overrides: vec![],
        }
    }

    pub fn with_overrides(&mut self, overrides: Vec<Override>) {
        self.overrides = overrides;
    }

    pub fn get_section_names(&self) -> Vec<String> {
        self.raw_value()
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

    fn override_values(&self, value: &mut Value) {
        for o in self.overrides.iter() {
            let sub_value = get_sub_value_by_keys(value, &o.keys);
            *sub_value = o.value.clone();
        }
    }

    fn raw_value(&self) -> Value {
        serde_yaml::from_str::<Value>(&self.contents).unwrap_or(Value::Null)
    }

    fn value(&self) -> Value {
        let value = serde_yaml::from_str::<Value>(&self.contents);
        match value {
            Ok(mut value) => {
                self.override_values(&mut value);
                value
            }
            Err(_) => {
                assert!(self.overrides.is_empty());
                Value::Null
            }
        }
    }

    pub fn contents(&self) -> String {
        serde_yaml::to_string(&self.value()).unwrap()
    }

    pub(super) fn extract_parameter_struct<T: Parameters>(&mut self) -> T {
        self.value()
            .get(T::name())
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
                        T::name()
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

    use super::Override;
    use super::ParameterFileContents;
    use crate::named::Named;

    #[test]
    fn r#override() {
        #[derive(Deserialize, Named)]
        #[name = "x"]
        struct X {
            a: usize,
            b: usize,
        }

        let o = Override {
            keys: vec!["x".into(), "a".into()],
            value: 5.into(),
        };
        let mut contents = ParameterFileContents {
            contents: "x:\n  a: 1\n  b: 2".into(),
            overrides: vec![o],
        };
        let x = contents.extract_parameter_struct::<X>();
        assert_eq!(x.a, 5);
        assert_eq!(x.b, 2);
    }
}
