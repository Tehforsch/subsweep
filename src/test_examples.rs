use std::process::Command;

use macro_utils::RaxiomManifest;

#[test]
#[ignore]
fn run_all_examples() {
    let manifest = RaxiomManifest::default();
    for example in manifest.examples() {
        let name = example.name.unwrap();
        if name == "mpi_test" {
            continue;
        }
        let required_features = example.required_features.join(",");
        let mut run_command = Command::new("cargo");
        run_command.args([
            "run",
            "-q",
            "--color",
            "always",
            "--example",
            &name,
            "--features",
            &required_features,
            "--",
            "simulation/final_time: 0.0 s",
            "example/num_particles: 50",
            "--headless",
            "true",
        ]);
        println!("Running example {}", &name);
        let output = run_command
            .output()
            .expect("Failed to run run command for example");
        if !output.status.success() {
            let stdout =
                || std::str::from_utf8(&output.stdout).expect("Failed to decode stdout as utf8");
            let stderr =
                || std::str::from_utf8(&output.stderr).expect("Failed to decode stderr as utf8");
            panic!(
                "Failed to run example {}.\nstdout:\n{}\nstderr:\n{}",
                &name,
                stdout(),
                stderr()
            );
        }
    }
}
