use serde::Deserialize;

#[derive(Deserialize, Default)]
pub(super) struct Parameters {
    #[serde(default)]
    pub show_local_quadtree: bool,
    #[serde(default)]
    pub show_remote_quadtree: bool,
    #[serde(default)]
    pub show_segment_extent: bool,
}
