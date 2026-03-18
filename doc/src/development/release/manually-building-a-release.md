# Manually Building a Release

## Native Release

To build release artifacts for distribution, run:
```sh
cargo ci distribution
```
The artifacts are placed under `target/ci/distribution/`.

You will need to install Docker or Podman on your system to perform the build.  
On Linux, if you go with Docker, you will need to add your user to the `docker` group.  
See the instructions here: <https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user> 

### Alternative Platform

If you want to build artifacts for a different platform, use the following:
```sh
cargo ci distribution --target armv7-unknown-linux-gnueabihf
```

The currently supported target platforms are:
* x86_64-unknown-linux-gnu
* armv7-unknown-linux-gnueabihf
* aarch64-unknown-linux-gnu



## Container

To build a Docker container of CARL and push it to the configured container registry:
```sh
cargo ci carl docker --publish
```
This will publish opendut-carl to `ghcr.io/eclipse-opendut/opendut-carl:x.y.z`.
The version defined in `opendut-carl/Cargo.toml` is used as container tag by default.

To do the same for EDGAR, this command can be used:
```sh
cargo ci edgar docker --publish
```


### Alternative Container Registry

Publish Docker container to another container registry than `ghcr.io`.

```sh
export OPENDUT_DOCKER_IMAGE_HOST=other-registry.example.net
export OPENDUT_DOCKER_IMAGE_NAMESPACE=opendut
cargo ci carl docker --publish --tag 0.1.1
```
This will publish opendut-carl to 'other-registry.example.net/opendut:opendut-carl:0.1.1'.
