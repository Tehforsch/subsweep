use mpi::request::Scope;
use mpi::request::WaitGuard;

use super::Rank;

pub trait WorldCommunicator<T> {
    fn blocking_send_vec(&mut self, rank: Rank, data: Vec<T>);
    fn receive_vec(&mut self, rank: Rank) -> Vec<T>;
    fn immediate_send_vec<'a, Sc: Scope<'a>>(
        &mut self,
        scope: Sc,
        rank: Rank,
        data: &'a [T],
    ) -> Option<WaitGuard<'a, [T], Sc>>;
}
