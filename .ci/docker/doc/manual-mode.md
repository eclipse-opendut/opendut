# Manual mode

Manually start containers from the command line.
All the commands are run from the repository root.

## Start containers

    ```sh
    docker compose -f .ci/docker/carl/docker-compose.yml --env-file .env up -d
    docker compose -f .ci/docker/edgar/docker-compose.yml --env-file .env up -d
    docker compose -f .ci/docker/firefox/docker-compose.yml --env-file .env up -d
    ```

## Environment variables

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
