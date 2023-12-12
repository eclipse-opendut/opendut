# Firefox in a container

This is a docker image for running firefox in a container. 
Following features are included:
- Resolves container names (no DNS manipulation needed)
- Custom certificate authorities are automatically trusted (no need to import those unsafe authorities to your browser)

## Usage

* Start container `docker compose up -d`
* Open remote session to firefox in docker your browser via [link](http://localhost:3000).
* Open url in remote session:
  * https://carl
  * http://netbird-ui
  * http://keycloak
