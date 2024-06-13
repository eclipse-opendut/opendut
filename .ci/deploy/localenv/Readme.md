# Local test environment

This directory contains everything to run a OpenDuT server locally for testing purposes.

## Description
The local test environment exposes its services to the intranet in contrast to the test environment (only for development purposes).

* It can be accessed by other devices on the same network.
* Can be used to test the server with real peers.
* Requires to run on the host system when the turn server (coturn) is used to interconnect peers that do not reach each other.


## Getting started


### Using vagrant

* Start the local test environment using vagrant.
```shell
export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
export VAGRANT_DOTFILE_PATH=$OPENDUT_REPO_ROOT/.vagrant
export VAGRANT_VAGRANTFILE=$OPENDUT_REPO_ROOT/.ci/deploy/localenv/Vagrantfile

# exposing ports 80 and 443 requires root privileges
export OPENDUT_EXPOSE_PRIVILEGED_PORTS=true
vagrant up
```
* Destroy the local test environment using vagrant (requires same environment variables as above).
```shell
vagrant destroy
```

* Remove secrets
```shell
rm .ci/deploy/localenv/secrets/.env
```

### Using docker compose

* Start the local test environment using docker compose.
```shell
# configure project path
export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
# start provisioning and create .env file
docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/provision/docker-compose.yml up --build
# start the environment
docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/.env up --detach --build
```

* Destroy the local test environment using docker compose.
```shell
docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml down --volumes
```

### Modify /etc/hosts

Add the following lines to the `/etc/hosts` file on the host system to access the services from the intranet.
Substitute the IP address with the of the host system when the ports are exposed.
```shell
192.168.56.9 opendut.local
192.168.56.9 auth.opendut.local
192.168.56.9 netbird.opendut.local
192.168.56.9 netbird-api.opendut.local
192.168.56.9 signal.opendut.local
192.168.56.9 carl.opendut.local

```
