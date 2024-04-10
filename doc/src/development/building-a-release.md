# Building a Release

To build release artifacts for distribution, run:
```sh
cargo ci distribution
```
The artifacts are placed under `target/ci/distribution/`.

To build a docker container of CARL and push it to the configured docker registry:
```sh
cargo ci carl docker --publish
```
This will publish opendut-carl to `ghcr.io/eclipse-opendut/opendut-carl:x.y.z`.
The version defined in `opendut-carl/Cargo.toml` is used as docker tag by default.

## Alternative platform

If you want to build artifacts for a different platform, use the following:
```sh
cargo ci distribution --target armv7-unknown-linux-gnueabihf
```

The currently supported target platforms are:
* x86_64-unknown-linux-gnu
* armv7-unknown-linux-gnueabihf
* aarch64-unknown-linux-gnu


## Alternative docker registry

Publish docker container to another container registry than `ghcr.io`.

```sh
export OPENDUT_DOCKER_IMAGE_HOST=other-registry.example.net
export OPENDUT_DOCKER_IMAGE_NAMESPACE=opendut
cargo ci carl docker --publish --tag 0.1.1
```
This will publish opendut-carl to 'other-registry.example.net/opendut:opendut-carl:0.1.1'.
