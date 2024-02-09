#### Use virtual machine for development


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
OPENDUT_CARL_VPN_NETBIRD_URL=http://192.168.56.10/api
OPENDUT_CARL_VPN_NETBIRD_HTTPS_ONLY=false
OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET=<dynamic_api_secret>
OPENDUT_CARL_VPN_NETBIRD_AUTH_TYPE=personal-access-token
OPENDUT_CARL_VPN_NETBIRD_AUTH_HEADER=Authorization
```
* You may also place the toml (also printed from the `carl-config` command) file in a special configuration file on your host at ``~/.config/opendut/carl/config.toml``.
* Use the environment variables in the run configuration for CARL
    * Command on the **host**: `cargo run --package opendut-carl --bin opendut-carl` 
* Or start CARL in your IDE of choice and add the environment variables to the run configuration.
