# Running EDGAR in a Docker container

You can run EDGAR in a Docker container by using the Docker Compose file defined here: <https://github.com/eclipse-opendut/opendut/blob/development/.ci/docker/edgar/docker-compose.yml>

Follow the instructions at the top of the file for running it.

Mind that this Docker container currently only supports Ethernet clusters.
For CAN support, this issue needs to be resolved: <https://github.com/eclipse-opendut/opendut/issues/486>
