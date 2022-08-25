pub trait CollectiveCommunicator<T> {
    fn all_gather(&mut self, send: &T) -> Vec<T>;
}

pub trait SumCommunicator<T> {
    fn collective_sum(&mut self, send: &T) -> T;
}
