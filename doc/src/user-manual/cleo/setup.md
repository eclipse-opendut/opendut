# Setup for CLEO

- Download the opendut-cleo binary for your target from the openDuT GitHub project: https://github.com/eclipse-opendut/opendut/releases
- Unpack the archive on your target system.
- Add a configuration file
`/etc/opendut/cleo.toml` (Linux)
and configure at least the CARL host+port.
The possible configuration values and their defaults can be seen here:  
```toml
 {{#include ../../../../opendut-cleo/cleo.toml}}
```

## Download CLEO from CARL
It is also possible to download CLEO from one of CARLs endpoints. The downloaded file contains the binary for CLEO for the requested architecture,
the necessary certificate file, as well as a setup script.

The archive can be requested at `https://{CARL-HOST}/api/cleo/{architecture}/download`.

Available architectures are:
- x86_64-unknown-linux-gnu
- armv7-unknown-linux-gnueabihf
- aarch64-unknown-linux-gnu

This might be the go-to way, if you want to use CLEO in your pipeline. 
Once downloaded, extract the files with the command `tar xvf opendut-cleo-{architecture}.tar.gz`. It will then be extracted into
the folder which is the current work directory. You might want to use another directory of your choice.

The script used to run CLEO will not set the environment variables for CLIENT_ID and CLIENT_SECRET. This has to be done by the users manually.
This can easily be done by entering the following commands:
````
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ID={{ CLIENT ID VARIABLE }} 
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET={{ CLIENT SECRET VARIABLE }} 
````
These two variables can be obtained by logging in to Keycloak.

The tarball contains the `cleo-cli.sh` shell script. When executed it starts CLEO after setting the
following environment variables:
````
OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SCOPES
OPENDUT_CLEO_NETWORK_TLS_DOMAIN_NAME_OVERRIDE
OPENDUT_CLEO_NETWORK_TLS_CA
OPENDUT_CLEO_NETWORK_CARL_HOST
OPENDUT_CLEO_NETWORK_CARL_PORT
OPENDUT_CLEO_NETWORK_OIDC_ENABLED
OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL
SSL_CERT_FILE
````

`SSL_CERT_FILE` is a mandatory environment variable for the current state of the implementation and has the same value as the 
`OPENDUT_CLEO_NETWORK_TLS_CA`. This might change in the future. 

Using CLEO with parameters works by adding the parameters when executing the script, e.g.:
````
./cleo-cli.sh list peers
````

### TL;DR
1. Download archive from `https://{CARL-HOST}/api/cleo/{architecture}/download`
2. Extract `tar xvf opendut-cleo-{architecture}.tar.gz`
3. Add two environment variable `export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ID={{ CLIENT ID VARIABLE }}` and `export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET={{ CLIENT SECRET VARIABLE }}`
4. Execute `cleo-cli.sh` with parameters

## Additional notes
- The CA certificate to be provided for CLEO depends on the used certificate authority used on server side for CARL.
