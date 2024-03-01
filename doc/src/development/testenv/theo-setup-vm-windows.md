# Setup THEO on Windows

This guide will help you set up THEO on Windows.

## Requirements
The following instructions use chocolatey to install the required software. 
If you don't have chocolatey installed, you can find the installation instructions [here](https://chocolatey.org/install).
You may also install the required software manually.

* Install vagrant and virtualbox
    ```sh
    choco install -y vagrant virtualbox
    ```
* Install git and configure git to respect line endings
    ```sh
    choco install git.install --params "'/GitAndUnixToolsOnPath /WindowsTerminal /NoAutoCrlf'"
    ```

> **Warning**
> If you already have installed git, you may need to reconfigure it to respect line endings.
> If you already have checked out the repository without this setting, you need to do it again.

  * Redo git configuration
    ```sh
    git config --global core.autocrlf false
    ```

* Create or check if an ssh key pair is present in `~/.ssh/id_rsa`
  ```sh
  mkdir -p ~/.ssh
  ssh-keygen -t rsa -b 4096 -C "opendut-vm" -f ~/.ssh/id_rsa
  ```

## Setup virtual machine

* Add the following environment variables to point vagrant to the vagrant file
    ```sh
    export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
    export VAGRANT_DOTFILE_PATH=$OPENDUT_REPO_ROOT/.vagrant
    export VAGRANT_VAGRANTFILE=$OPENDUT_REPO_ROOT/.ci/docker/Vagrantfile
    ```
* Set up the vagrant box (following commands were tested in git-bash)
```sh
vagrant up
```


> **Info**
> If the virtual machine is not allowed to create or use a private network you may disable it by setting the environment variable `OPENDUT_DISABLE_PRIVATE_NETWORK=true`.

* Connect to the virtual machine via ssh (requires the environment variables)
```sh
vagrant ssh
```

## Additional notes
You may want to configure a http proxy or a custom certificate authority. 
Details are in the **Advance usage** section.
