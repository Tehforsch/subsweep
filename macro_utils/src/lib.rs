// Adapted and modified from bevy_macro_utils

use std::{path::Path, env};

use cargo_toml::{Manifest, Product};

pub struct RaxiomManifest {
    pub manifest: Manifest,
}

impl RaxiomManifest {
    pub fn examples(&self) -> Vec<Product> {
        self.manifest.example.clone()
    }
}

impl Default for RaxiomManifest {
    fn default() -> Self {
        let manifest_path = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
        let manifest = Manifest::from_path(&manifest_path).unwrap_or_else(|e| {
            panic!(
                "Failed to parse manifest file at {:?}: {}",
                &manifest_path, e
            )
        });
        Self {
            manifest
        }
    }
}
