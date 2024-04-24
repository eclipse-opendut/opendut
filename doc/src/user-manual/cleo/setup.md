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

## Additional notes
- The CA certificate to be provided for CLEO depends on the used certificate authority used on server side for CARL.
