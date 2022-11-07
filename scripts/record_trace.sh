rm output.tracy
tracy-capture -o output.tracy&
cargo run --release --features bevy/trace_tracy --example "$@"
