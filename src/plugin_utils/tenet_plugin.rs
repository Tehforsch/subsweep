use super::Simulation;
use crate::named::Named;

pub trait TenetPlugin: Named {
    fn should_build(&self, _sim: &Simulation) -> bool {
        true
    }
    fn build_everywhere(&self, _sim: &mut Simulation) {}
    fn build_on_main_rank(&self, _sim: &mut Simulation) {}
    fn build_on_other_ranks(&self, _sim: &mut Simulation) {}
    fn build_once_everywhere(&self, _sim: &mut Simulation) {}
    fn build_once_on_main_rank(&self, _sim: &mut Simulation) {}
    fn build_once_on_other_ranks(&self, _sim: &mut Simulation) {}
}
