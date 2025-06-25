# Changelog

Notable changes to this project are documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Breaking Changes
* The ClusterConfiguration has been renamed to ClusterDescriptor for consistency. (The user *describes* what they want.)

### Added
* CARL: A CLI has been added for viewing the contents of the database.
* CLEO now has a `--verbose` flag to enable more detailed logging.
* CARL, EDGAR and CLEO now log when there's version incompatibilities between them.

### Changed
* Localenv: opendut-carl-postgres container removed, might need manual removal when updating.
* CLEO: Deleting a resource which doesn't exist does not anymore produce an error code. 
  If you need the previous behaviour, use the `--error-when-missing` flag.
* EDGAR: No longer re-creates the peer network configuration in case one peer in the cluster comes back online. [#343](https://github.com/eclipse-opendut/opendut/issues/343)

* Monitoring: Scraping logs of Docker containers is now done with Grafana Alloy, since Promtail has been deprecated. (Thanks to [@brtmax](https://github.com/brtmax)!)
* Updated keycloak version in testenv and localenv to version `26.2.5`. Backup of the keycloak database is recommended.

### Fixed
* The web-UI now shows peers, clusters and devices sorted by name.
* Deleting clusters in the web-UI works again.
* Resolved issue where the telemetry logs where not transmitted because a lock on the Confidential Client could not be acquired. [#347](https://github.com/eclipse-opendut/opendut/issues/347)

## [0.6.0] - 2025-04-22

### Breaking Changes
* The database backend was swapped out for a Key-Value Store.  
  Before upgrading, create a backup. Then run the following on the target host within the openDuT repository:
  ```sh
  git fetch
  git checkout v0.6.0

  docker stop opendut-carl

  export OPENDUT_REPO_ROOT=$(git rev-parse --show-toplevel)
  docker compose --file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/docker-compose.yml --env-file ${OPENDUT_REPO_ROOT:-.}/.ci/deploy/localenv/data/secrets/.env run --env OPENDUT_CARL_POSTGRES_MIGRATION=true --env OPENDUT_CARL_IMAGE_VERSION=0.6.0 carl
  ```
  Then trigger a rollout as normal. If you're using the Ansible deployment, make sure to increment the version there before doing so.

  If you plan to upgrade to a future/newer version, you cannot skip upgrading to v0.6.0 as the migration code is only contained herein.

### Added
* Localenv: Add environment variable `OPENDUT_CARL_IMAGE_VERSION` to override the CARL image in use.
* When a user deletes a peer that is still connected, the peer will be disconnected.
* CLEO now offers to await the online status of a set of peers with the `await` command.

### Fixed
* Updating a cluster descriptor does not anymore remove its deployment status.
* Fixed deleting peer group and setup key in the API for NetBird.
* THEO: The test environment is now destroyed at the end, allowing subsequent runs.


## [0.5.1] - 2025-02-28

### Added
* Backups include PKI and other credentials now
* Backups include NetBird Management database now

### Fixed
* The web-UI now loads much faster when there are many peers.
* CLEO now outputs network interfaces as a list when running `opendut-cleo list peers --output=json`.
* Localenv deployment provisioning scripts persist PKI secrets on host now.


## [0.5.0] - 2025-02-12

### Added
* Automation of the backend and edge rollout.
* Automation of database backups.

### Changed
* Major upgrade of frontend framework and clean-up of authentication flow.

### Fixed
* The monitoring dashboard now shows data from all services.
* Logs from third-party libraries don't get filtered out anymore.


## [0.4.0] - 2024-12-20

<details>
<summary>Breaking Changes</summary>

* The LocalEnv deployment now uses different volume names.
  Make sure to back up your database and restore the backup after the upgrade.
</details>

### Added
* CLEO now comes with a new subcommand `opendut-cleo apply`. 
  You may load cluster and peer configurations from a YAML file, similar to how resources are loaded with `kubectl apply` in Kubernetes. 
  For more information, see [Applying Configuration Files](https://opendut.eclipse.dev/book/user-manual/cleo/commands.html#applying-configuration-files).

* A monitoring dashboard is now available in the deployment environment at `https://monitoring.opendut.local`.

### Fixed
* A major upgrade of the networking libraries has been completed.  
  This affects HTTP and gRPC, server- and client-side usage, as well as the OpenTelemetry monitoring.

* CARL does not anymore send duplicate Executor and Ethernet bridge name configurations to EDGAR when re-deploying a cluster.  
  This may have caused EDGAR to repeatedly delete and recreate these.


## [0.3.1] - 2024-11-15

### Fixed
* Restarting EDGAR while a cluster is deployed doesn't lead to an invalid state anymore.
* CARL doesn't forget about Ethernet bridges and executors anymore, when sending the configuration to a reconnecting EDGAR.
* EDGAR Setup now loads plugins correctly.


## [0.3.0] - 2024-10-28

<details>
<summary>Breaking Changes</summary>
* The API for listing peers on the PeerMessagingBroker has been removed.
</details>

### Added
* CARL can now persist its state into a database.
* EDGAR Setup now has support for plugins, which can perform hardware- or use-case specific setup tasks.

### Changed
* EDGAR Setup now prompts whether to overwrite a mismatched configuration, when used interactively.
* The NetBird server and client was updated to 0.28.9.

### Fixed
* EDGAR Service does not require root permissions anymore, if CAN is not used.

### Known Issues
* Stopping an EDGAR that has a cluster deployed, does not undeploy the cluster, therefore blocking other EDGARs in the cluster.


## [0.2.0] - 2024-09-10

<details>
<summary>Breaking Changes</summary>

### CARL API
* The API for listing peers on the PeerMessagingBroker is now marked as deprecated.

### Operations

* An additional configuration value needs to be passed to CARL. You can do so, for example, via environment variable:
```
OPENDUT_CARL_NETWORK_OIDC_CLIENT_ISSUER_ADMIN_URL=https://keycloak/admin/realms/opendut/
```
The value has to be your Keycloak's Admin URL.

* The environment variable for the Keycloak database's password was renamed from `POSTGRES_PASSWORD` to `KEYCLOAK_POSTGRES_PASSWORD`.
* An additional password environment variable needs to be provided called `CARL_POSTGRES_PASSWORD`.
</details>

### Added
* CARL can now require clients to be authenticated.
* A download button for CLEO and EDGAR has been added in the LEA web-UI.
* LEA and CLEO show when a peer or device is already used in a cluster.
* You can now configure frequently used CAN parameters in LEA and CLEO.
* Setup-Strings can now be copied to the clipboard in LEA.

### Changed
* The health of Clusters marked as deployed is now displayed as yellow in LEA.  
  This is to reflect that determining the actual cluster state is not yet implemented.
* It's not anymore possible to configure deployment of a peer into two clusters.  
  This was never supported to begin with, but the UIs didn't prevent it.
* Various quality-of-life improvements.

### Fixed
* Generating a Setup-String now works for peers which had previously been set up.


## [0.1.0] - 2024-06-05

### Added
* Client credentials added to peer setup

<details>
<summary>Development</summary>

### Test environment
Notable changes to the test environment are documented in this section.
Changes to the test environment may require re-provisioning the virtual machine.

#### Added
* New administrative privileges for keycloak client opendut-carl-client
* Added linux-generic package to opendut-vm (keeps vcan module up-to-date when kernel is updated)
</details>
