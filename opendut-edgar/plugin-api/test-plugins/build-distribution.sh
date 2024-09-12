#!/bin/sh
set -ex

cargo install cargo-component --locked
cargo component build --release

rm -r target/distribution/ || true  #ignore errors if directory doesn't exist
mkdir --parents target/distribution/test-plugins/

cp target/wasm32-wasip1/release/*.wasm target/distribution/test-plugins/
cp plugins.txt target/distribution/test-plugins/

tar --directory=target/distribution/ --create --gzip --file=target/distribution/test-plugins.tar.gz  test-plugins/

echo "Distribution placed at: target/distribution/test-plugins.tar.gz"
