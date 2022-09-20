use bevy::prelude::*;

use super::get_parameters;
use super::run_once;
use crate::communication::WorldRank;
use crate::named::Named;

pub trait TenetPlugin: Named {
    fn build_everywhere(&self, _app: &mut App) {}
    fn build_on_main_rank(&self, _app: &mut App) {}
    fn build_on_other_ranks(&self, _app: &mut App) {}
    fn build_once_everywhere(&self, _app: &mut App) {}
    fn build_once_on_main_rank(&self, _app: &mut App) {}
    fn build_once_on_other_ranks(&self, _app: &mut App) {}
}

pub(super) struct IntoPlugin<T>(T);

impl<T> From<T> for IntoPlugin<T> {
    fn from(t: T) -> Self {
        IntoPlugin(t)
    }
}

impl<T: TenetPlugin + Sync + Send + 'static> Plugin for IntoPlugin<T> {
    fn build(&self, app: &mut App) {
        self.0.build_everywhere(app);
        if get_parameters::<WorldRank>(app).is_main() {
            self.0.build_on_main_rank(app);
        } else {
            self.0.build_on_other_ranks(app);
        }
        run_once::<T>(app, |app| {
            self.0.build_once_everywhere(app);
            if get_parameters::<WorldRank>(app).is_main() {
                self.0.build_once_on_main_rank(app);
            } else {
                self.0.build_once_on_other_ranks(app);
            }
        });
    }
}
