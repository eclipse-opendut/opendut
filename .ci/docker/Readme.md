# Docker test environment

This is a docker test environment for the examples. It is started with docker-compose:
- carl
- edgar
- dev container includes rust build tools
- firefox container for UI testing 
  - includes certificate authorities and is running in headless mode
  - is running in same network as carl and edgar (working DNS resolution!)


## Getting started

* Install docker compose (see https://docs.docker.com/compose/install/linux/)


* Make distribution
```sh
cargo make distribution
```
* Start containers
```
./.ci/docker/theo.rs start
```
* Start firefox container
```
docker compose -f .ci/docker/firefox/docker-compose.yml --env-file .env up -d
```
* Open http://netbird-ui and create API key and setup key for peer group `testenv`

* Other possible urls in remote session:
  * https://carl
  * http://netbird-ui
  * http://keycloak


## Manual testing
* Commands are run from git repository root directory.

* Create distribution
    ```sh
    cargo make distribution
    ```

* Build carl image (other images are built automatically)
    ```sh
    docker compose -f .ci/docker/carl/docker-compose.yml build
    ```
    Carl adds artifact during build time. The container includes the artifact.
    The other images are not published and therefore mount the artifacts from filesystem which is more dynamic.

## Start containers


    ```sh
    docker compose -f .ci/docker/carl/docker-compose.yml --env-file .env up -d
    docker compose -f .ci/docker/edgar/docker-compose.yml --env-file .env up -d
    docker compose -f .ci/docker/firefox/docker-compose.yml --env-file .env up -d
    ```

## Special dev container

* Prepare container environment variables
    ```bash
    echo PUID=$(id -u) >> .env
    echo PGID=$(id -g) >> .env
    echo PUSER=$(id -un) >> .env
    echo PGROUP=$(id -gn) >> .env
    echo DOCKER_GID=$(cut -d: -f3 < <(getent group docker)) >> .env
    echo OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel) >> .env
    ```
* Build dev container
    ```bash
    docker compose -f .ci/docker/dev/docker-compose.yml --env-file .env build
    docker compose -f .ci/docker/dev/docker-compose.yml --env-file .env up
    ```
