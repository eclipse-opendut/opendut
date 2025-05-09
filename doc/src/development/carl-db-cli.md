# CARL DB CLI

CARL contains a small CLI to allow viewing the contents of the database.  
This CLI can be used with `opendut-carl db`.

However, it requires that the CARL service is not currently running,
since only one process can access the database at a time.

To resolve this, a stripped-down version of the CARL container can be started.

LocalEnv:
```sh
docker stop opendut-carl

docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/.env run --name=carl-db -ti --rm --entrypoint="" carl bash

docker start opendut-carl
```

TestEnv:
```sh
docker stop opendut-carl

docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/docker/carl/docker-compose.yml --env-file ${OPENDUT_REPO_ROOT:-.}/.env --env-file ${OPENDUT_REPO_ROOT:-.}/.env-theo run --name=carl-db -ti --rm --entrypoint="" carl bash

docker start opendut-carl
```
