use bevy::prelude::Component;

use super::count_by_dir::CountByDir;

#[derive(Component)]
pub struct Site {
    pub num_missing_upwind: CountByDir,
}
