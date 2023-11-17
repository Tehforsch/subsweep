use core::env;
use std::fs::File;

use serde::Serialize;
use subsweep::parameters::OutputParameters;

#[derive(Serialize)]
struct BuildInfo {
    timestamp: &'static str,
    features: &'static str,
    opt_level: &'static str,
    target_triple: &'static str,

    commit_message: &'static str,
    commit_timestamp: &'static str,
    branch: &'static str,
    commit: &'static str,
}
pub fn emit_build_information(output_params: &OutputParameters) {
    let build_info = BuildInfo {
        timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
        features: env!("VERGEN_CARGO_FEATURES"),
        opt_level: env!("VERGEN_CARGO_OPT_LEVEL"),
        target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
        commit_message: env!("VERGEN_GIT_COMMIT_MESSAGE"),
        commit_timestamp: env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
        branch: env!("VERGEN_GIT_BRANCH"),
        commit: env!("VERGEN_GIT_SHA"),
    };
    let path = output_params.output_dir.join("build_info.yml");
    serde_yaml::to_writer(File::create(path).unwrap(), &build_info).unwrap();
}
