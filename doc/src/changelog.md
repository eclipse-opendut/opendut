# Changelog

Notable changes to this project are documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.1

### Fixed
* Restarting EDGAR while a cluster is deployed doesn't lead to an invalid state anymore.
* CARL doesn't forget about Ethernet bridges and executors anymore, when sending the configuration to a reconnecting EDGAR.
* EDGAR Setup now loads plugins correctly.


## 0.3.0

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


## 0.2.0

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


## 0.1.0

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
