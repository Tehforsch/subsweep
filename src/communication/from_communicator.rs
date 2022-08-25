pub trait FromCommunicator<C> {
    fn from_communicator(communicator: C) -> Self;
}
