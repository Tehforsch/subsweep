use std::collections::HashMap;

use bevy::prelude::debug;
use derive_traits::RaxiomParameters;
use serde_yaml::Mapping;
use serde_yaml::Value;

#[derive(Debug, Clone)]
pub struct Override {
    pub section: String,
    pub keys: Vec<String>,
    pub value: Value,
}

pub struct ParameterFileContents {
    sections: HashMap<String, Value>,
    overrides: Vec<Override>,
}

fn insert_overrides(value: &mut Value, overrides: &[Override]) {
    for o in overrides.iter() {
        // This value specifies the entire section. Return.
        if o.keys.is_empty() {
            *value = o.value.clone();
            return;
        } else {
            insert_sublevel_override(value, o);
        }
    }
}

fn insert_sublevel_override(value: &mut Value, o: &Override) {
    set_sublevel_value_by_keys(value, &o.keys, o.value.clone());
}

fn extract_from_default<T: RaxiomParameters>(overrides: &[Override]) -> T {
    let section_name = T::unwrap_section_name();
    debug!(
        "Parameter section missing for '{}', assuming defaults",
        section_name
    );
    let get_value = || {
        let mut value = Value::Mapping(Mapping::default());
        insert_overrides(&mut value, overrides);
        value
    };
    match serde_yaml::from_value::<T>(get_value()) {
        Ok(obj) => obj,
        Err(e) => {
            let value = get_value();
            if !value.is_mapping() {
                panic!("Failed to parse section {}: {}", section_name, e);
            }
            let values_overriden = !value.as_mapping().unwrap().is_empty();
            if values_overriden {
                panic!(
                    "Failed to parse required section {}. Error: {}.",
                    section_name, e
                );
            } else {
                panic!(
                    "Required section {} not present in parameter file.",
                    section_name
                );
            }
        }
    }
}

fn extract_from_section<T: RaxiomParameters>(
    overrides: &[Override],
    section_value: &mut Value,
) -> T {
    // The following is a workaround for deserializing a serde_yaml::Value,
    // which fails when visiting dimensionless quantities (which will be interpreted as floats)
    insert_overrides(section_value, overrides);
    serde_yaml::from_str(&serde_yaml::to_string(section_value).unwrap()).unwrap_or_else(|err| {
        panic!(
            "Failed to read parameter file section \"{:?}\": \n{}",
            T::section_name(),
            err
        )
    })
}

/// Constructs a map of the form
/// key1: key2: key3: ... key_n: Value
/// If keys is empty, returns value
fn construct_sub_mapping(keys: &[String], value: Value) -> Value {
    if keys.is_empty() {
        value
    } else {
        let mut map = Mapping::default();
        map.insert(
            Value::String(keys[0].clone()),
            construct_sub_mapping(&keys[1..], value),
        );
        Value::Mapping(map)
    }
}

fn set_sublevel_value_by_keys(value: &mut Value, keys: &[String], target_value: Value) {
    if keys.is_empty() {
        *value = target_value;
    } else {
        let mapping = value.as_mapping_mut();
        match mapping {
            Some(mapping) => match mapping.get_mut(&keys[0]) {
                Some(key) => set_sublevel_value_by_keys(key, &keys[1..], target_value),
                None => {
                    mapping.insert(
                        Value::String(keys[0].clone()),
                        construct_sub_mapping(&keys[1..], target_value),
                    );
                }
            },
            None => unreachable!(),
        }
    }
}

impl ParameterFileContents {
    pub fn new(contents: String) -> Self {
        let sections = serde_yaml::from_str(&contents)
            .map(|val: Value| {
                val.as_mapping()
                    .expect("Could not parse parameter file as mapping")
                    .iter()
                    .map(|(k, v)| (k.as_str().unwrap().to_owned(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            sections,
            overrides: vec![],
        }
    }

    pub fn with_overrides(&mut self, overrides: Vec<Override>) {
        self.overrides = overrides;
    }

    pub fn get_section_names(&self) -> impl Iterator<Item = &String> {
        self.sections.keys()
    }

    fn get_overrides_for_section(
        &self,
        section_name: String,
    ) -> impl Iterator<Item = Override> + '_ {
        self.overrides
            .iter()
            .filter(move |o| o.section == section_name)
            .cloned()
    }

    pub fn contents(&self) -> String {
        let mut map = serde_yaml::Mapping::default();
        for (name, value) in self.sections.iter() {
            map.insert(Value::String(name.into()), value.clone());
        }
        serde_yaml::to_string(&map).unwrap()
    }

    pub(super) fn extract_parameter_struct<T: RaxiomParameters>(&mut self) -> T {
        let section_name = T::unwrap_section_name();
        let overrides_this_section = self
            .get_overrides_for_section(section_name.to_owned())
            .collect::<Vec<_>>();
        match self.sections.get_mut(section_name) {
            Some(section_value) => extract_from_section(&overrides_this_section, section_value),
            None => {
                let extracted = extract_from_default::<T>(&overrides_this_section);
                self.sections.insert(
                    section_name.to_string(),
                    serde_yaml::to_value(&extracted).unwrap(),
                );
                extracted
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use derive_custom::raxiom_parameters;

    use super::Override;
    use super::ParameterFileContents;

    #[raxiom_parameters("x")]
    struct X {
        a: usize,
        b: usize,
    }

    #[test]
    fn r#override() {
        let mut contents = ParameterFileContents::new("x:\n  a: 1\n  b: 2".into());
        contents.with_overrides(vec![Override {
            section: "x".into(),
            keys: vec!["a".into()],
            value: 5.into(),
        }]);
        let x = contents.extract_parameter_struct::<X>();
        assert_eq!(x.a, 5);
        assert_eq!(x.b, 2);
    }

    #[test]
    fn r#override_in_omitted_section() {
        let mut contents = ParameterFileContents::new("".into());
        contents.with_overrides(vec![
            Override {
                section: "x".into(),
                keys: vec!["b".into()],
                value: 6.into(),
            },
            Override {
                section: "x".into(),
                keys: vec!["a".into()],
                value: 5.into(),
            },
        ]);
        let x = contents.extract_parameter_struct::<X>();
        assert_eq!(x.a, 5);
        assert_eq!(x.b, 6);
    }

    #[test]
    fn r#override_omitted_section() {
        #[raxiom_parameters("s")]
        struct Section(i32);

        let mut contents = ParameterFileContents::new("".into());
        contents.with_overrides(vec![Override {
            section: "s".into(),
            keys: vec![],
            value: 5.into(),
        }]);
        let section = contents.extract_parameter_struct::<Section>();
        assert_eq!(section.0, 5);
    }

    #[test]
    fn r#override_omitted_field() {
        #[raxiom_parameters("y")]
        struct Y {
            #[serde(default)]
            a: usize,
            b: usize,
        }

        let mut contents = ParameterFileContents::new("y:\n  b: 2".into());
        contents.with_overrides(vec![Override {
            section: "y".into(),
            keys: vec!["a".into()],
            value: 5.into(),
        }]);
        let y = contents.extract_parameter_struct::<Y>();
        assert_eq!(y.a, 5);
        assert_eq!(y.b, 2);
    }
}
