set -x

sources=( target/release/dupfinder )

function _package_main {
    output_file=$1
    packaging_dir=packaging/$output_file
    mkdir -pv $packaging_dir
    echo sources $sources
    echo output_file $output_file
    cp -t $packaging_dir/ ${sources[@]}
    tar -C packaging -cf - $output_file | zstd -T10 -16 -o "$output_file.tar.zst" -
    tar -C packaging -cf - $output_file | xz -T10 -6 -c > "$output_file.tar.xz"
    rm -rfv packaging
}