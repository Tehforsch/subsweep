use derive_custom::raxiom_parameters;

#[raxiom_parameters("sweep")]
pub struct SweepParameters {
    pub num_directions: usize,
}
