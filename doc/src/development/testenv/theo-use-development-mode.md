# Use virtual machine for development


![OpenDuT-VM](..%2F..%2F..%2F..%2Fresources%2Fdiagrams%2Fopendut-vm-development.drawio.svg)


* Start vagrant on **host**: `cargo theo vagrant up`
* Connect to virtual machine from **host**: `cargo theo vagrant ssh`
* Start developer test mode in **opendut-vm**: `cargo theo dev start`

* Once keycloak and netbird are provisioned, generate run configuration for CARL
  in **opendut-vm**:
  `cargo theo dev carl-config`
    * which should give an output similar to the following:
```
OPENDUT_CARL_NETWORK_REMOTE_HOST=carl
OPENDUT_CARL_NETWORK_REMOTE_PORT=443
OPENDUT_CARL_VPN_ENABLED=true
OPENDUT_CARL_VPN_KIND=netbird
OPENDUT_CARL_VPN_NETBIRD_URL=https://192.168.56.10/api
OPENDUT_CARL_VPN_NETBIRD_CA=<ca_certificate_filepath>
OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET=<dynamic_api_secret>
OPENDUT_CARL_VPN_NETBIRD_AUTH_TYPE=personal-access-token
OPENDUT_CARL_VPN_NETBIRD_AUTH_HEADER=Authorization
```
* You may also use the toml configuration (also printed from the `carl-config` command) file in a special configuration file on your host at ``~/.config/opendut/carl/config.toml``.
* Use the environment variables in the run configuration for CARL
    * Run CARL on the **host**: `cargo ci carl run` 
    * Run LEA on the **host**: `cargo ci lea run` 
* Or start CARL in your IDE of choice and add the environment variables to the run configuration.

## Use CLEO

When using CLEO in your IDE or generally on the host, 
the address for keycloak needs to be overridden, as well as the address for CARL.

```
# Environment variables to use  CARL on host
export OPENDUT_CLEO_NETWORK_CARL_HOST=localhost
export OPENDUT_CLEO_NETWORK_CARL_PORT=8080
# Environment variable to use keycloak in test environment
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL=http://localhost:8081/realms/opendut/
cargo ci cleo run -- list peers
```
