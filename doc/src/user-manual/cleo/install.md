# Installing CLEO

- Download the opendut-cleo binary for your target from the openDuT Github project https://github.com/eclipse-opendut/opendut/releases
- Unpack the binary on your target system
- Add a CLEO configuration file
`/etc/opendut/cleo.toml` (Linux)
with the following content and adjust it to your particular openDuT setup:
```` 
[network]
carl.host = "carl.opendut.local"
carl.port = 443

[network.tls]
ca = "/etc/opendut/tls/ca.pem"
domain.name.override = ""


[network.oidc]
enabled = false

[network.oidc.client]
id = "opendut-cleo-client"
issuer_url = "https://keycloak/realms/opendut"
scopes = "openid,profile,email,roles,groups"
secret = "<tbd>"
````

## Additional notes
- The ca certificate to be provided for CLEO depends on the used certificate authority used on server side for CARL
