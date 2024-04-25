# Setup of CARL

- To configure CARL, you can create a configuration file under `/etc/opendut/carl.toml`.  
The possible configuration values and their defaults can be seen here:  
```toml
{{#include ../../../../opendut-carl/carl.toml}}
```

## Additional notes
- We're currently working on automating the setup of CARL, since a complete setup requires additional services.  
  You can find a rough setup in the repository under `.ci/docker/`, which you may be able to adapt.
