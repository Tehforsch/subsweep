use derive_custom::raxiom_parameters;

/// Settings that have an impact on the simulation performance.
#[raxiom_parameters("performance")]
pub struct PerformanceParameters {
    /// The batch size for parallel iterations. Low batch sizes
    /// increase parallelism at the cost of additional overhead needed
    /// for spawning the futures, whereas large batch sizes prevent
    /// parallelization but reduce overhead.
    /// A value of None will force effectively serial iterations.
    pub batch_size: Option<usize>,
}

impl Default for PerformanceParameters {
    fn default() -> Self {
        Self {
            batch_size: Some(1000),
        }
    }
}

impl PerformanceParameters {
    pub(crate) fn batch_size(&self) -> usize {
        // Using a really large value effectively turns off any kind of parallelism
        self.batch_size.unwrap_or(usize::MAX)
    }
}
