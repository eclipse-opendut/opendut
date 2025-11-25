# Setup of CARL

Currently, our setup is automated via Docker Compose.  

If you want to use CARL and its components on a separate machine, i.e. a Raspberry PI 
or any other machine, this guide will show all necessary steps, to get CARL up and running.

![Hosting Environment](../img/opendut-vm-edgar-test-setup.drawio.svg)

1. Install Git, if not already installed and checkout the openDuT repository:
    ```shell
    git clone https://github.com/eclipse-opendut/opendut.git
    ```

2. Install Docker and Docker Compose v2, e.g. on Debian-based operating systems:
   ```shell
   sudo apt install docker.io docker-compose-v2
   ```

3. Optional: Change the docker image location CARL should be pulled from in `.ci/deploy/localenv/docker-compose.yml`.
  By default, CARL is pulled from `ghcr.io`.

4. Set `/etc/hosts` file:
Add the following lines to the `/etc/hosts` file on the host system to access the services from the local network.
This assumes that the system, where OpenDuT was deployed, has the IP address `192.168.56.10`
    ```shell
    192.168.56.10 opendut.local
    192.168.56.10 auth.opendut.local
    192.168.56.10 netbird.opendut.local
    192.168.56.10 netbird-api.opendut.local
    192.168.56.10 signal.opendut.local
    192.168.56.10 carl.opendut.local
    192.168.56.10 nginx-webdav.opendut.local
    192.168.56.10 opentelemetry.opendut.local
    192.168.56.10 monitoring.opendut.local
    ```

5. Start the local test environment using Docker Compose.
    ```shell
    # configure project path
    export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
    # start provisioning and create .env file
    docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/.env.development up --build provision-secrets
    # delete old secrets, if they exist, ensuring they are not copied to a subdirectory
    rm -rf ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/
    # copy the created secrets to the host, ensuring they are readable for the current user
    docker cp opendut-provision-secrets:/provision/ ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/
    # start the environment
    docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml --env-file .ci/deploy/localenv/.env.development --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/.env up --detach --build
    ```
    In this step secrets are going to be created and all containers are getting started. \
    The secrets which were created during the first `docker compose` command can be found in `.ci/deploy/localenv/data/secrets/.env`.
    Domain names are configured in environment file `.env.development`.

If everything worked and is up and running, you can follow the [EDGAR Setup Guide](../edgar/setup.md).


## Shutdown the environment

* Stop the local test environment using docker compose.
```shell
docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml down
```

* Destroy the local test environment using docker compose.
```shell
docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml down --volumes
```

## Configuration
- You can configure the log level of CARL via the environment variable `OPENDUT_LOG`.  
  For example, to only show INFO logging and above, set it as `OPENDUT_LOG=info`.  
  For more fine-grained control, see the documentation here: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives>
- The general configuration of CARL can be set via environment variables or by manually creating a configuration file under `/etc/opendut/carl.toml`.  
  The environment variables use the TOML keys in the configuration file, joined by underscores and in capital letters.
  For example, to configure the `network.bind.host` use the environment variable `NETWORK_BIND_HOST`.  
  The possible configuration values and their defaults can be seen here:  
```toml
{{#include ../../../../opendut-carl/carl.toml}}
```
