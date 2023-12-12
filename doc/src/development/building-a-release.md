# Building a Release

To build release artifacts for distribution, run:
```sh
cargo make distribution
```
The artifacts are placed under `target/ci/distribution/`.


## Alternative platform

If you want to build artifacts for a different platform, use the following:
```sh
cargo make distribution --env TARGET="armv7-unknown-linux-gnueabihf"
```

The currently supported target platforms are:
* x86_64-unknown-linux-gnu
* armv7-unknown-linux-gnueabihf
* aarch64-unknown-linux-gnu
