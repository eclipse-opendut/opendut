# User Manual for CLEO

CLEO is a CLI tool to create/read/update/delete resources in CARL.

By using a terminal you will be able to configure your resources via CLEO.

CLEO can currently access the following resources:
- Cluster configurations
- Cluster deployments 
- Peers
- Devices (DuTs)
- Container executors

Every resource can be created, listed, described and deleted.
Some have additional features such as an option to generate a setup-key or search through them.

In general, CLEO offers a `help` command to display usage information about a command. Just use `opendut-cleo help` or `opendut-cleo <subcommand> --help`.
