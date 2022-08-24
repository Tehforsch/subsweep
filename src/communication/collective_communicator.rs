pub enum Operation {
    Sum,
}

pub trait CollectiveCommunicator<T> {
    fn all_gather(&mut self, send: &T) -> Vec<T>;
    fn all_reduce(&mut self, send: &T, operation: Operation) -> T;
}
