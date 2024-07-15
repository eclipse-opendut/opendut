# Setup of CARL

Currently, our setup is automated via Docker Compose.  
See the folder `.ci/deploy/localenv/` in our repository for details. 

## Configuration
- To configure CARL, you can create a configuration file under `/etc/opendut/carl.toml`.  
The possible configuration values and their defaults can be seen here:  
```toml
{{#include ../../../../opendut-carl/carl.toml}}
```
