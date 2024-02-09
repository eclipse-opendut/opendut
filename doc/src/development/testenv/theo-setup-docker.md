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

