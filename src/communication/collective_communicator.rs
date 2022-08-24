pub enum Operation {
    Sum,
}

pub trait CollectiveCommunicator<T> {
    fn all_gather(&self, send: T) -> Vec<T>;
    fn all_reduce(&self, send: T, operation: Operation) -> T;
}
