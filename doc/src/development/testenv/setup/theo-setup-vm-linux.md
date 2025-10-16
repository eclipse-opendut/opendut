# Setup THEO on Linux in VM

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
  If the package is not available for your distribution, you may need to add a package repository as described here: <https://developer.hashicorp.com/vagrant/install#linux>

* Install VirtualBox (see https://www.virtualbox.org)
  ```sh
  sudo apt install virtualbox
  ```
  To get a version compatible with Vagrant, you may need to add the APT repository as described here: <https://www.virtualbox.org/wiki/Linux_Downloads#Debian-basedLinuxdistributions>

* Create or check if an SSH key pair is present in `~/.ssh/id_rsa`
  ```sh
  mkdir -p ~/.ssh
  ssh-keygen -t rsa -b 4096 -C "opendut-vm" -f ~/.ssh/id_rsa
  ```
  You can create a symlink to your existing key as well.

## Setup virtual machine

* Either via cargo:
  ```sh
  cargo theo vagrant up
  ```
* Login to the virtual machine
  ```sh
  cargo theo vagrant ssh
  ```

