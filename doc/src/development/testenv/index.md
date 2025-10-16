# Test Environment

openDuT can be tricky to test, as it needs to modify the operating system to function and supports networking in a distributed setup.  
To aid in this, we offer a virtualized test environment for development.  
This test environment is set up with the help of a command line tool called `theo`.
THEO stands for **Test Harness Environment Operator**.

It is recommended to start everything in a virtual machine, but you may also start the service on the host with `docker compose` if applicable.
Setup of the virtual machine is done with Vagrant, Virtualbox and Ansible.
The following services are included in docker:
- CARL (OpenDuT backend software)
- EDGAR (OpenDuT edge software)
- firefox container for UI testing (accessible via http://localhost:3000) 
  - includes certificate authorities and is running in headless mode
  - is running in same network as carl and edgar (working DNS resolution!)
- [NetBird](https://github.com/netbirdio/netbird) Third-party software that provides WireGuard based VPN
- [Keycloak](https://www.keycloak.org/) Third-party software that provides authentication and authorization

## Operational modes

There are two ways of operation for the test environment:

### Test mode

Run everything in Docker (Either on your host or preferable in a virtual machine).
You may use the **OpenDuT Browser** to access the services.
The OpenDuT Browser is a web browser running in a docker container in the same network as the other services.
All certificates are pre-installed and the browser is running in headless mode.
It is accessible from your host via http://localhost:3000.

![OpenDuT-VM](img/opendut-vm-user.drawio.svg)

### Development mode

Run CARL on the host in your development environment of choice and the rest in Docker.
In this case there is a proxy running in the docker environment. 
It works as a drop-in replacement for CARL in the docker environment, which is forwarding the traffic to CARL running in an integrated development environment on the host.

![OpenDuT-VM](img/opendut-vm-development.drawio.svg)

## Getting started

Follow the [setup steps](setup/index.md).
Then you may start the test environment in the virtual machine or in plain docker.
* And use it in [test mode](usage/theo-use-test-mode.md)
* Or use it in [development mode](usage/theo-use-development-mode.md).
* If you want to build the project in the virtual machine you may also want to give it more [resources](advanced.md#give-the-virtual-machine-more-cpu-cores-and-more-memory) (cpu/memory).

There are some known issues with the test environment (most of them on Windows): 
* [known issues section](known-issues.md)



