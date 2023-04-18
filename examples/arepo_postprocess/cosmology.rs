use derive_custom::raxiom_parameters;

#[raxiom_parameters("cosmology")]
#[derive(Debug)]
#[serde(untagged)]
pub enum Cosmology {
    Cosmological { a: f64, h: f64 },
    NonCosmological,
}
