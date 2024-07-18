# THEO Setup in Vagrant

You may run all the containers in a virtual machine, using Vagrant.
This is the recommended way to run the test environment.
It will create a private network (subnet 192.168.56.0/24).
The virtual machine itself has the IP address: `192.168.56.10`.
The docker network has the IP subnet: `192.168.32.0/24`.
Make sure those network addresses are not occupied or in conflict with other networks accessible from your machine.

## Requirements

* Install Vagrant

  *Ubuntu / Debian*
  ```sh
  sudo apt install vagrant
  ```
  On most other Linux distributions, the package is called `vagrant`.
* Install VirtualBox (see https://www.virtualbox.org)
  ```sh
  sudo apt install virtualbox
  ```
* Create or check if an ssh key pair is present in `~/.ssh/id_rsa`
  ```sh
  mkdir -p ~/.ssh
  ssh-keygen -t rsa -b 4096 -C "opendut-vm" -f ~/.ssh/id_rsa
  ```

## Setup virtual machine

* Either via cargo:
  ```
  cargo theo vagrant up
  ```
* Login to the virtual machine
  ```
  cargo theo vagrant ssh
  ```

> **Warning**
> Within the VM the rust target directory is overridden to `/home/vagrant/rust-target` to avoid hard linking issues.
> When running cargo within the VM, output will be placed in this directory!

* Ensure a distribution of openDuT is present
    * By either creating one yourself (on the host)
      ```sh
      cargo ci distribution
      ```
    * Or by copying one to the target directory `target/ci/distribution/x86_64-unknown-linux-gnu/`
      ```sh
      mkdir -p target/ci/distribution/x86_64-unknown-linux-gnu/
      ```

* Start test environment
  ```
  cargo theo testenv start
  ```
