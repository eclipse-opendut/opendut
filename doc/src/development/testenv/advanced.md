# Advanced Usage

Run vagrant commands directly instead of through THEO:
* or directly via Vagrant's cli (bash commands run from the root of the repository):
  ```
  export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
  export VAGRANT_DOTFILE_PATH=$OPENDUT_REPO_ROOT/.vagrant
  export VAGRANT_VAGRANTFILE=$OPENDUT_REPO_ROOT/.ci/docker/Vagrantfile
  vagrant up
  ```
* provision vagrant with desktop environment
  ```
  ANSIBLE_SKIP_TAGS="" vagrant provision
  ```

## Cross compile THEO for Windows on Linux

```
cross build --release --target x86_64-pc-windows-gnu --bin opendut-theo
# will place binary here
target/x86_64-pc-windows-gnu/release/opendut-theo.exe
```

## Custom Certificate Authority
This section shall provide information on how to
provision the virtual machine when running behind an intercepting http proxy.
This is also used in the docker containers to trust the custom certificate authority.
All certificate authorities matching the following path will be trusted in the docker container:
`./resources/development/tls/*-ca.pem`.

The following step needs to be done before provisioning the virtual machine.
* Place certificate authority file here: `resources/development/tls/custom-ca.crt`
