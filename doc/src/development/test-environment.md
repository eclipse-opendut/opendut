# Test Environment

This is a test environment for openDuT.
The test environment is set up with the help of a command line tool called `theo`.
THEO stands for **Test Harness Environment Operator**.

It is recommended to start everything in a virtual machine, but you may also start the service on the host with `docker compose` if applicable.
Setup of the virtual machine is done with Vagrant, Virtualbox and Ansible.
The following services are included in docker:
- carl
- edgar
- firefox container for UI testing (accessible via http://localhost:3000) 
  - includes certificate authorities and is running in headless mode
  - is running in same network as carl and edgar (working DNS resolution!)
- netbird
- keycloak

## Operational modes

There are two ways of operation for the test environment:

### Test mode

Run everything in Docker (Either on your host or preferable in a virtual machine).
You may use the **OpenDuT Browser** to access the services.
The OpenDuT Browser is a web browser running in a docker container in the same network as the other services.
All certificates are pre-installed and the browser is running in headless mode.
It is accessible from your host via http://localhost:3000.

![OpenDuT-VM](..%2F..%2F..%2Fresources%2Fdiagrams%2Fopendut-vm-user.drawio.svg)

### Development mode

Run CARL on the host in your development environment of choice and the rest in Docker.
In this case there is a proxy running in the docker environment. 
It works as a drop-in replacement for CARL in the docker environment, which is forwarding the traffic to CARL running in an integrated development environment on the host.

![OpenDuT-VM](..%2F..%2F..%2Fresources%2Fdiagrams%2Fopendut-vm-development.drawio.svg)

## Getting started

Set up the virtual machine
* on [Windows](./testenv/theo-setup-vm-windows.md)
* on [Linux](./testenv/theo-setup-vm-linux.md)

Then you may start the test environment in the virtual machine.
* And use it in [test mode](./testenv/theo-use-test-mode.md)
* Or use it in [development mode](./testenv/theo-use-development-mode.md).

There are some known issues with the test environment (most of them on Windows): 
* [known issues section](./testenv/known-issues.md)


## Start testing

Once you have set up and started the test environment, you may start testing the services.

### User interface

The **OpenDuT Browser** is a web browser running in a docker container.
It is based on KasmVNC base image which allows containerized desktop applications from a web browser.
A port forwarding is in place to access the browser from your host.
It has all the necessary certificates pre-installed and is running in headless mode.
You may use this **OpenDuT Browser** to access the services.

* Open following address in your browser: http://localhost:3000
* Usernames for test environment:
  * LEA: opendut:opendut
  * Keycloak: admin:admin123456
  * Netbird: netbird:netbird
* Services with user interface:
  * https://carl
  * http://netbird-dashboard
  * https://keycloak

