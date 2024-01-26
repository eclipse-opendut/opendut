pub(crate) fn print_carl_config_toml(netbird_management_url: &str, netbird_token: &str) {
    println!(r#"
# CARL configuration environment variables for development.
[network]
bind.host = "0.0.0.0"
bind.port = 8080
remote.host = "carl"
remote.port = 443

[network.tls]
certificate = "resources/development/tls/carl.pem"
key = "resources/development/tls/carl.key"
[network.tls.ca]
certificate = "resources/development/tls/insecure-development-ca.pem"

[network.oidc]
enabled = true

[network.oidc.lea]
client_id = "opendut-lea-client"
issuer_url = "https://keycloak/realms/opendut"
scopes = "openid,profile,email"

[serve]
ui.directory = "./opendut-lea/dist/"

[vpn]
enabled = true
kind = "netbird"

[vpn.netbird]
url = "{}"
https.only = false
auth.type = "personal-access-token"
auth.secret = "{}"

"#, netbird_management_url, netbird_token);

}

pub(crate) fn print_carl_config_env(netbird_management_url: &str, netbird_token: &str) {
    println!(r#"
# CARL configuration toml for development.
OPENDUT_CARL_NETWORK_BIND_HOST=0.0.0.0
OPENDUT_CARL_NETWORK_BIND_PORT=8080
OPENDUT_CARL_NETWORK_REMOTE_HOST=carl
OPENDUT_CARL_NETWORK_REMOTE_PORT=443

OPENDUT_CARL_NETWORK_TLS_CERTIFICATE=resources/development/tls/carl.pem
OPENDUT_CARL_NETWORK_TLS_KEY=resources/development/tls/carl.key
OPENDUT_CARL_NETWORK_TLS_CA_CERTIFICATE=resources/development/tls/insecure-development-ca.pem
OPENDUT_CARL_NETWORK_OIDC_ENABLED=true
OPENDUT_CARL_NETWORK_OIDC_LEA_CLIENT_ID=opendut-lea-client
OPENDUT_CARL_NETWORK_OIDC_LEA_ISSUER_URL=https://keycloak/realms/opendut
OPENDUT_CARL_NETWORK_OIDC_LEA_SCOPES=openid,profile,email
OPENDUT_CARL_SERVE_UI_DIRECTORY=./opendut-lea/dist/

OPENDUT_CARL_VPN_ENABLED=true
OPENDUT_CARL_VPN_KIND=netbird
OPENDUT_CARL_VPN_NETBIRD_URL={}
OPENDUT_CARL_VPN_NETBIRD_HTTPS_ONLY=false
OPENDUT_CARL_VPN_NETBIRD_AUTH_TYPE=personal-access-token
OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET={}

        "#, netbird_management_url, netbird_token);
}
