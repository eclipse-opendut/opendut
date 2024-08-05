# Changelog

Notable changes to this project are documented in this file.  
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

* tbd.

## 0.2.0

<details>
<summary>Breaking Changes</summary>

### Operations

* An additional configuration value needs to be passed to CARL. You can do so, for example, via environment variable:
```
OPENDUT_CARL_NETWORK_OIDC_CLIENT_ISSUER_ADMIN_URL=https://keycloak/admin/realms/opendut/
```
The value has to be your Keycloak's Admin URL.
</details>

### Added
* CARL can now require clients to be authenticated.

### Changed
* The health of Clusters marked as deployed is now displayed as yellow in LEA.  
  This is to reflect that determining the actual cluster state is not yet implemented.

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
