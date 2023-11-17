use std::error::Error;

use vergen::EmitBuilder;

#[derive(Debug)]
struct MyErr;
impl Error for MyErr {}

impl std::fmt::Display for MyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hi {}", 3)
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    // Emit the instructions
    EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .emit()?;
    Ok(())
}
