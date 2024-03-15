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

It adds the GitHub enterprise extension to the image:
* [github-enterprise-identity-provider](https://github.com/thomasdarimont/keycloak-extension-playground)

## Commands of interest

* List realm roles: `kcadm get-roles -r opendut`
* Add realm role to client: `kcadm add-roles -r opendut --uusername service-account-<CLIENT_ID> --rolename <ROLE_NAME>`

