# Running EDGAR in a Docker container

You can experimentally run EDGAR in a Docker container by using the Docker Compose file defined here: <https://github.com/eclipse-opendut/opendut/blob/development/.ci/docker/edgar/docker-compose.yml>

Follow the instructions at the top of the file for running it.


## CAN

If you want to use CAN with this Docker container, you need to make sure that the kernel modules `vcan` and `can_gw` are loaded on the host system (outside of the container).
The `can_gw` module has to be loaded with the parameter `max_hops=2`.

Make sure that the modules are loaded anew after a reboot, i.e. don't just load them via `modprobe`, but rather create configuration files, so that the host system loads them automatically.
