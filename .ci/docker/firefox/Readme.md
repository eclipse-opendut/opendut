# Firefox in a container

This is a docker image for running firefox in a container. 
Following features are included:
- Resolves container names (no DNS manipulation needed, no need to add hosts to your host file)
- Custom certificate authorities are automatically trusted (no need to import those unsafe authorities to your browser)
- Custom firefox profile (no need to configure firefox every time you start it)

## Usage

* Start container `docker compose up -d`
* Open remote session to firefox in docker your browser via [link](http://localhost:3000).
* Open url in remote session:
  * https://carl
  * https://netbird-dashboard
  * https://keycloak


## GitHub container registry

Added a copy to GHCR due to connectivity issues with the original source.
* [GitHub GHCR Documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
* Change visibility of the package to public in the [settings](https://github.com/orgs/eclipse-opendut/packages/container/firefox/settings).
* Manage GitHub actions access in the settings to allow access to the package from the opendut repository.
* Update container in GitHub container registry
```shell
VERSION=124.0.1-r0-ls156
docker pull lscr.io/linuxserver/firefox:$VERSION
docker tag lscr.io/linuxserver/firefox:$VERSION ghcr.io/eclipse-opendut/firefox:$VERSION
docker login ghcr.io -u <USERNAME>
docker push ghcr.io/eclipse-opendut/firefox:$VERSION
```
