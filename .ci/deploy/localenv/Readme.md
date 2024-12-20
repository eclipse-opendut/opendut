# Local test environment

This directory contains everything to run a OpenDuT server locally for testing purposes.

## Description
The local test environment exposes its services to the intranet in contrast to the test environment (only for development purposes).

* It can be accessed by other devices on the same network.
* Can be used to test the server with real peers.
* Requires to run on the host system when the turn server (coturn) is used to interconnect peers that do not reach each other.


## Getting started

See the official setup documentation [here](https://opendut.eclipse.dev/book/user-manual/carl/setup.html) for details.

### Using vagrant

These steps are for developers to test the docker-compose setup in a virtual machine. 

* Start the local test environment using vagrant.
```shell
export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
export VAGRANT_DOTFILE_PATH=$OPENDUT_REPO_ROOT/.vagrant
export VAGRANT_VAGRANTFILE=$OPENDUT_REPO_ROOT/.ci/deploy/localenv/Vagrantfile
vagrant up
```
* Destroy the local test environment using vagrant (requires same environment variables as above).
```shell
vagrant destroy
```

* Remove secrets
```shell
rm -r .ci/deploy/localenv/data/secrets/
```

* Proxy notes for testing in virtual machine

Testing localenv in a virtual machine behind corporate proxy
```shell
source .ci/deploy/localenv/data/provision/proxy.sh http://192.168.42.1:3128
```
