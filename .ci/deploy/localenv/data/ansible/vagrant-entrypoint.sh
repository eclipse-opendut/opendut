#!/usr/bin/env bash

cd /vagrant || { echo Could not change directory to /vagrant; exit 1; }

docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/provision/docker-compose.yml up --build
docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/.env up --detach --build

echo "All containers started. You may observe the containers by connecting to the VM:"
echo "vagrant ssh"

echo "The following secrets were created:"
cat .ci/deploy/localenv/data/secrets/.env

echo -e "\n---------------------\n"
echo "docker ps"
echo "cd /vagrant"
echo "docker compose logs --tail=0 --follow"
