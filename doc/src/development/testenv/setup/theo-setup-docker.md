# Setup THEO in Docker

## Requirements

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


* Add your user into the `docker` group, to be allowed to use Docker commands without root permissions: <https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user>


* Update /etc/hosts
    ```shell
    127.0.0.1 opendut.local
    127.0.0.1 auth.opendut.local
    127.0.0.1 netbird-api.opendut.local
    127.0.0.1 netbird-relay.opendut.local
    127.0.0.1 signal.opendut.local
    127.0.0.1 nginx-webdav.opendut.local
    127.0.0.1 opentelemetry.opendut.local
    127.0.0.1 monitoring.opendut.local
    ```
