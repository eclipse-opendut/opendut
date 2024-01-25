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
  * http://netbird-ui
  * https://keycloak
