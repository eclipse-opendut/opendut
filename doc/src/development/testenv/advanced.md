# Advanced Usage

## Use vagrant directly

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

## Re-provision the virtual machine

This is recommended after potentially breaking changes to the virtual machine.

* Following command will re-run the ansible playbook to re-provision the virtual machine.
Run from host:
```shell
cargo theo vagrant provision
```
* Destroy test environment and re-create it, run within the virtual machine:
```shell
cargo theo vagrant ssh
cargo theo testenv destroy
cargo theo testenv start
```

## Cross compile THEO for Windows on Linux

```
cross build --release --target x86_64-pc-windows-gnu --bin opendut-theo
# will place binary here
target/x86_64-pc-windows-gnu/release/opendut-theo.exe
```

## Proxy configuration
In case you are working behind a http proxy, you need additional steps to get the test environment up and running.
The following steps pick up just _before_ you start up the virtual machine with `vagrant up`.
A list of all domains used by the test environment is reflected in the proxy shell script:
`.ci/docker/vagrant/proxy.sh`.
It is important to note that the proxy address used shall be accessible from the host while provisioning and within
the virtual machine.

If you have a proxy server on your localhost you need to make this in two steps:
* Use proxy on your localhost
  * Configure vagrant to use the proxy localhost.
    ```shell
    # proxy configuration script, adjust to your needs
    source .ci/docker/vagrant/proxy.sh http://localhost:3128
    ```
  * Install proxy plugin for vagrant
    ```shell
    vagrant plugin install vagrant-proxyconf
    ```
  * Then starting the VM without provisioning it. 
    This should create the vagrant network interface with network range 192.168.56.0/24.
    ```
    vagrant up --no-provision
    ```
* Use proxy on private network address 192.168.56.1
  * Make sure this address is allowing access to the internet:
    ```
    curl --max-time 2 --connect-timeout 1 --proxy http://192.168.56.1:3128 google.de
    ```
  * Redo the proxy configuration using the address of the host within the virtual machine's private network:
    ```shell
    # proxy configuration script, adjust to your needs
    source .ci/docker/vagrant/proxy.sh http://192.168.56.1:3128
    ```
  * Reapply the configuration to the VM
    ```shell
    $ vagrant up --provision
    Bringing machine 'opendut-vm' up with 'virtualbox' provider...
    ==> opendut-vm: Configuring proxy for Apt...
    ==> opendut-vm: Configuring proxy for Docker...
    ==> opendut-vm: Configuring proxy environment variables...
    ==> opendut-vm: Configuring proxy for Git...
    ==> opendut-vm: Machine not provisioned because `--no-provision` is specified.
    ```

* Unset all proxy configuration for testing purposes (non-permanent setting in the shell)
    ```shell
    unset http_proxy https_proxy no_proxy HTTP_PROXY HTTPS_PROXY NO_PROXY
    ```

* You may also set the docker proxy configuration in your environment manually:
  * `~/.docker/config.json`
    ```json
    {
      "proxies": {
        "default": {
          "httpProxy": "http://x.x.x.x:3128",
          "httpsProxy": "http://x.x.x.x:3128",
          "noProxy": "localhost,127.0.0.1,netbird-management,netbird-dashboard,netbird-signal,netbird-coturn,keycloak,edgar-leader,edgar-*,carl,192.168.0.0/16"
        }
      }
    }
    ```
  * `/etc/docker/daemon.json`
    ```json
    {
      "proxies": {
        "http-proxy": "http://x.x.x.x:3128",
        "https-proxy": "http://x.x.x.x:3128",
        "no-proxy": "localhost,127.0.0.1,netbird-management,netbird-dashboard,netbird-signal,netbird-coturn,keycloak,edgar-leader,edgar-*,carl,192.168.0.0/16"
      }
    }
    ```

## Custom root certificate authority
This section shall provide information on how to
provision the virtual machine when running behind an intercepting http proxy.
This is also used in the docker containers to trust the custom certificate authority.
All certificate authorities matching the following path will be trusted in the docker container:
`./resources/development/tls/*-ca.pem`.

The following steps need to be done before provisioning the virtual machine.
* Place certificate authority file here: `resources/development/tls/custom-ca.crt`
* Optionally, disable private network definition of vagrant, if this causes errors.
```sh
export CUSTOM_ROOT_CA=resources/development/tls/custom-ca.pem
export OPENDUT_DISABLE_PRIVATE_NETWORK=true  # optional
vagrant provision
```
