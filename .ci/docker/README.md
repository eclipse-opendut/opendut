# Docker test environment

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
* or directly via Vagrant (from the root of the repository):
  ```
  export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
  VAGRANT_VAGRANTFILE=.ci/docker/Vagrantfile vagrant up
  ```

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
  * http://keycloak


