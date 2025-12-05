# Configuration

This is a high-level guide for how to configure openDuT applications, including useful tips and tricks.


## Setting values
The configuration can be set via environment variables or by manually creating a configuration file, e.g. under `/etc/opendut/carl.toml`.

The environment variables use the TOML keys from the configuration file in capital letters, joined by underscores and prefixed by the application name.
For example, to configure `network.bind.host` in CARL, use the environment variable `OPENDUT_CARL_NETWORK_BIND_HOST`.

See the end of this file for the configuration file format.


## TLS certificates
When configuring a TLS certificate/key, you can provide either a file path or the text of the certificate directly.
The latter is useful in particular when working with environment variables.

You can provide separate CA certificates for individual backend services, namely OpenTelemetry, NetBird and OIDC.
If you do not do so, the CA certificate from `network.tls.ca` will be used as the default.  


## Log level
You can configure the log level via the environment variable `OPENDUT_LOG`.  
For example, to only show INFO logging and above, set it as `OPENDUT_LOG=info`.  
For more fine-grained control, see the documentation here: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives>


## Configuration file format
These are example configurations of the different applications, together with their default values.

### CARL
```toml
{{#include ../../../opendut-carl/carl.toml}}
```

### EDGAR
```toml
{{#include ../../../opendut-edgar/edgar.toml}}
```

### CLEO
```toml
{{#include ../../../opendut-cleo/cleo.toml}}
```
