# Keycloak docker image

This image is based on the official Keycloak docker image. 
* [Keycloak docker getting started](https://www.keycloak.org/getting-started/getting-started-docker)
* [Keycloak docker documentation](https://www.keycloak.org/server/containers)
* [Redhat ca-certificates](https://www.redhat.com/sysadmin/ca-certificates-cli)
    * Update the ca-certificates in the image
    * Test in container:
    ```
    docker run --rm --network opendut_network --name=keycloak_test -d --entrypoint="" registry.access.redhat.com/ubi9 sleep infinity
    ```

## GitHub Enterprise Support

Keycloak now supports GitHub Enterprise authentication natively without requiring additional plugins:
* [GitHub Identity Provider Documentation](https://www.keycloak.org/docs/latest/server_admin/#github)
* Configure via Admin Console: Identity Providers → Add Provider → GitHub

## Commands of interest

* List realm roles: `kcadm get-roles -r opendut`
* Add realm role to client: `kcadm add-roles -r opendut --uusername service-account-<CLIENT_ID> --rolename <ROLE_NAME>`

## Upgrading

* [GitHub releases page](https://github.com/keycloak/keycloak/releases)
* [Migration guide and breaking changes](https://www.keycloak.org/docs/latest/upgrading/index.html#migrating-to-26-2-0)
* [Compatible postgres version](https://www.keycloak.org/server/db)
* [Keycloak docker images](https://quay.io/repository/keycloak/keycloak?tab=tags)
* [Postgres docker images](https://hub.docker.com/_/postgres/tags)

## Keycloak versions

* [22.0.2](https://www.keycloak.org/2023/09/keycloak-2202-released)
* [26.0.0](https://www.keycloak.org/2024/10/keycloak-2600-released)
* [26.2.5](https://www.keycloak.org/2025/05/keycloak-2625-released)

## Upgrading notes 26.2.5

* Update configuration [parameters](https://www.keycloak.org/server/all-config#category-proxy)
  * `KC_PROXY` removed, set `KC_PROXY_HEADERS=xforwarded` for usage behind traefik
* Updated `kcadm.sh` command line parameter
* Updated build arguments for optimized keycloak [startup](https://www.keycloak.org/server/configuration#_optimize_the_keycloak_startup)

## Configuration Features

This build enables the following Keycloak features:
* `token-exchange` - Token exchange for service-to-service communication
* `scripts` - JavaScript-based authentication flows
* `preview` - Preview features for testing

## Database Configuration

Configured for PostgreSQL with health checks and metrics enabled for monitoring integration.
