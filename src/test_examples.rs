use std::env;
use std::path::Path;
use std::process::Command;

use cargo_toml::Manifest;

#[test]
#[ignore]
fn run_all_examples() {
    let manifest_path = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let manifest = Manifest::from_path(&manifest_path).unwrap_or_else(|e| {
        panic!(
            "Failed to parse manifest file at {:?}: {}",
            &manifest_path, e
        )
    });
    for example in manifest.example {
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
