cargo test --no-default-features && cargo test && cargo mpirun --np 2 --example mpi_test --features mpi_test
