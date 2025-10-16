# Use virtual machine for development

This mode is used to test a debug or development builds of OpenDuT.
Instead of running CARL in docker, a proxy is used to forward requests to CARL running on the host.

![OpenDuT-VM](../img/opendut-vm-development.drawio.svg)

Prepare the test environment to run in development mode:
* Start vagrant on **host**: `cargo theo vagrant up`
* Connect to virtual machine from **host**: `cargo theo vagrant ssh`
* Start developer test mode in **opendut-vm**: `cargo theo dev start`
* Update `/etc/hosts` on **host** to resolve addresses to the virtual machine, see [setup].

Use a different command to start the applications configured for the test environment:
* Run CARL on the **host**: `cargo theo dev carl`
* Run LEA on the **host**: `cargo lea`

* Run EDGAR on the machine that you started the test environment: 
  ```
  cargo theo dev edgar-shell
  ```
* Use either distribution build or debug build of EDGAR:
  ```
  cat create_edgar_service.sh   # to see how to start EDGAR step-by-step and how to use CLEO to manage devices and peers
  ./create_edgar_service.sh     # to start distribution build of EDGAR
  
  # notes for manual start
  root@338b6f728a0b:/opt# type -a opendut-edgar 
  opendut-edgar is /usr/local/opendut/bin/distribution/opendut-edgar  # extracted distribution build
  
  root@338b6f728a0b:/opt# type -a opendut-cleo
  opendut-cleo is /usr/local/opendut/bin/distribution/opendut-cleo    # distribution build
  opendut-cleo is /usr/local/opendut/bin/debug/opendut-cleo           # debug build mounted as volume from host
  ```

## Use CLEO

When using CLEO in your IDE or generally on the host, 
the address for keycloak needs to be overridden, as well as the address for CARL.

```
# Environment variables to use  CARL on host
export OPENDUT_CLEO_NETWORK_CARL_HOST=localhost
export OPENDUT_CLEO_NETWORK_CARL_PORT=8080
# Environment variable to use keycloak in test environment
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL=http://localhost:8081/realms/opendut/
cargo cleo list peers
```
