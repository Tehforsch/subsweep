if [[ $# != 1 ]]; then
    echo "Provide maximum number of cores available."
    exit 1
fi
cargo build --example mpi_performance --release
cargo_target_path=$(cargo metadata --format-version 1 | jq -r .target_directory)
binary="$cargo_target_path/release/examples/mpi_performance"
num_cores_final=$1
num_cores=1
while [[ 1 ]] ; do
    echo "NUM CORES: " $num_cores
    mpirun -n $num_cores $binary
    num_cores=$(( num_cores * 2 ))
    if [[ $num_cores -ge $num_cores_final ]]; then
        break
    fi
done

echo "NUM CORES: " $num_cores_final
mpirun -n $num_cores_final $binary
