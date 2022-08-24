pub trait CollectiveCommunicator<T> {
    fn all_gather(&self, send: T) -> Vec<T>;
}
