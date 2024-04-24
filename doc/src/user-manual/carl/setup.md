# Setup of CARL

- Download the opendut-carl binary from the openDuT GitHub project: https://github.com/eclipse-opendut/opendut/releases
- Unpack the archive on your target system, into `/opt/opendut-carl/`.

- Add a configuration file
`/etc/opendut/carl.toml` (Linux)
and configure as needed.
The possible configuration values and their defaults can be seen here:  
```toml
{{#include ../../../../opendut-carl/carl.toml}}
```
