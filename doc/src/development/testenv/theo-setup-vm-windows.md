# Setup THEO on Windows

This guide will help you set up THEO on Windows.

## Requirements
The following instructions use chocolatey to install the required software. 
If you don't have chocolatey installed, you can find the installation instructions [here](https://chocolatey.org/install).
You may also install the required software manually or e.g. use the Windows Package Manager [`winget`](https://learn.microsoft.com/en-us/windows/package-manager/) (Hashicorp.Vagrant, Oracle.VirtualBox, Git.Git).

* Install vagrant and virtualbox
    ```sh
    choco install -y vagrant virtualbox
    ```
* Install git and configure git to respect line endings
    ```sh
    choco install git.install --params "'/GitAndUnixToolsOnPath /WindowsTerminal'"
    ```


* Create or check if a ssh key pair is present in `~/.ssh/id_rsa`
  ```sh
  mkdir -p ~/.ssh
  ssh-keygen -t rsa -b 4096 -C "opendut-vm" -f ~/.ssh/id_rsa
  ```

> **Info**  
> Vagrant creates a VM which mounts a Windows file share on `/vagrant`, where the openDuT repository was cloned. The openDuT project contains bash scripts that would break if the end of line conversion to `crlf` on windows would happen.
> Therefore a [.gitattributes](https://git-scm.com/docs/gitattributes) file containing  
```*.sh text eol=lf```  
was added to the repository in order to make sure the bash scripts also keep the eol=`lf` when cloned on Windows.
> As an alternative, you may consider using the cloned opendut repo on the Windows host only for the vagrant VM setup part. For working with THEO, you can use the cloned opendut repository inside the Linux guest system instead (`/home/vagrant/opendut`).  

## Setup virtual machine

* Add the following environment variables to point vagrant to the vagrant file  
  Git Bash:
    ```sh
    export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
    export VAGRANT_DOTFILE_PATH=$OPENDUT_REPO_ROOT/.vagrant
    export VAGRANT_VAGRANTFILE=$OPENDUT_REPO_ROOT/.ci/deploy/opendut-vm/Vagrantfile
    ```
    PowerShell:
    ```powershell
    $env:OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
    $env:VAGRANT_DOTFILE_PATH="$env:OPENDUT_REPO_ROOT/.vagrant"
    $env:VAGRANT_VAGRANTFILE="$env:OPENDUT_REPO_ROOT/.ci/deploy/opendut-vm/Vagrantfile"
    ```
* Set up the vagrant box (following commands were tested in Git Bash and Powershell)
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
You may want to configure an http proxy or a custom certificate authority. 
Details are in the **Advance usage** section.
