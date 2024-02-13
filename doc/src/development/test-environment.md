# Test Environment

This is a Docker test environment for openDuT. It is started with docker-compose:
- carl
- edgar
- dev container includes rust build tools
- firefox container for UI testing 
  - includes certificate authorities and is running in headless mode
  - is running in same network as carl and edgar (working DNS resolution!)


## Getting started


### THEO Setup in Docker

#### Requirements

* Install Docker

   *Ubuntu / Debian*
   ```sh
   sudo apt install docker.io
   ```
   On most other Linux distributions, the package is called `docker`.


* Install Docker Compose v2

  *Ubuntu / Debian*
  ```sh
  sudo apt install docker-compose-v2
  ```
  Alternatively, see <https://docs.docker.com/compose/install/linux/>.

* Add your user into the `docker` group, to be allowed to use Docker commands without root permissions. (Mind that this has security implications.)
   ```sh
   sudo groupadd docker  # create `docker` group, if it does not exist
   sudo gpasswd --add $USER docker  # add your user to the `docker` group
   newgrp docker  # attempt to activate group without re-login
   ```
   You may need to log out your user account and log back in for this to take effect.


* Create a distribution of openDuT
```sh
cargo ci distribution
```

* Start containers
```
cargo theo testenv start
```

* Start edgar cluster
```
cargo theo testenv edgar start
```

### THEO Setup in Vagrant

You may run all of the above also in a virtual machine, using Vagrant.
It will create a private network (subnet 192.168.56.0/24).
The virtual machine itself has the IP address: `192.168.56.10`.
The docker network has the IP subnet: `192.168.32.0/24`.
Make sure those are not occupied.

#### Requirements

* Install Vagrant

  *Ubuntu / Debian*
  ```sh
  sudo apt install vagrant
  ```
  On most other Linux distributions, the package is called `vagrant`.
* Install VirtualBox (see https://www.virtualbox.org)

#### Start Vagrant

* Either via cargo:
  ```
  cargo theo vagrant up
  ```
* or directly via Vagrant's cli (bash commands run from the root of the repository):
  ```
  export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
  export VAGRANT_DOTFILE_PATH=$OPENDUT_REPO_ROOT/.vagrant
  export VAGRANT_VAGRANTFILE=$OPENDUT_REPO_ROOT/.ci/docker/Vagrantfile
  vagrant up
  ```
* provision vagrant with desktop environment
  ```
  ANSIBLE_SKIP_TAGS="" vagrant provision
  ```

#### Use virtual machine for development


* Start vagrant: `cargo theo vagrant up`
* Connect to virtual machine: `cargo theo vagrant ssh`
* Start developer test mode by either:
  * Running via cargo `cargo theo dev start` 
  * Reusing the debug build from the host if applicable (same target architecture): `./target/debug/opendut-theo dev start` 
* Once keycloak and netbird are provisioned, generate run configuration for CARL
  `cargo theo dev carl-config`
  * which should give you following output:
```
OPENDUT_CARL_NETWORK_REMOTE_HOST=carl
OPENDUT_CARL_NETWORK_REMOTE_PORT=443
OPENDUT_CARL_VPN_ENABLED=true
OPENDUT_CARL_VPN_KIND=netbird
OPENDUT_CARL_VPN_NETBIRD_URL=http://192.168.56.10/api
OPENDUT_CARL_VPN_NETBIRD_HTTPS_ONLY=false
OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET=<dynamic_api_secret>
OPENDUT_CARL_VPN_NETBIRD_AUTH_TYPE=personal-access-token
OPENDUT_CARL_VPN_NETBIRD_AUTH_HEADER=Authorization
```
* Use the environment variables in the run configuration for CARL 
  * Command: 'run --package opendut-carl --bin opendut-carl' 


##### Known issue
When running cargo tasks within the virtual machine, you may see following error:
```
warning: hard linking files in the incremental compilation cache failed. copying files instead. consider moving the cache directory to a file system which supports hard linking in session dir
```
This can be avoided by setting a different target directory for cargo, e.g.:
```
export CARGO_TARGET_DIR=$HOME/my-target
```

## Start testing

### User interface

Open following address in your browser:
  * docker mode: http://localhost:3000
  * vagrant mode: http://192.168.56.10:3000/
* Usernames for test environment:
  * Keycloak: admin:admin123456
  * Netbird: netbird:netbird
* Services with user interface:
  * https://carl
  * http://netbird-ui
  * https://keycloak


