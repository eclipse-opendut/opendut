use crate::util::CLEO_IDENTIFIER;
use config::Config;
use indoc::formatdoc;

pub struct CleoScript {
    pub carl_host: String,
    pub carl_port: u16,
    pub oidc_enabled: bool,
    pub issuer_url: String,
}

impl CleoScript {
    pub fn from_setting(settings: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            carl_host: settings.get_string("network.remote.host")?,
            carl_port: settings.get_int("network.remote.port")? as u16,
            oidc_enabled: settings.get_bool("network.oidc.enabled")?,
            issuer_url: settings.get_string("network.oidc.client.issuer.url")?,
        })
    }

    pub fn build_script(&self) -> String {

        let ca_certificate_file_name = super::CA_CERTIFICATE_FILE_NAME;
        let carl_host = &self.carl_host;
        let carl_port = self.carl_port;
        let oidc_enabled = self.oidc_enabled;
        let issuer_url = &self.issuer_url;
        let cleo_executable_name = CLEO_IDENTIFIER;

        formatdoc!(r#"#!/bin/bash

DIR_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
CERT_PATH=$DIR_PATH/{ca_certificate_file_name}

export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SCOPES=
export OPENDUT_CLEO_NETWORK_TLS_DOMAIN_NAME_OVERRIDE={carl_host}
export OPENDUT_CLEO_NETWORK_TLS_CA=$CERT_PATH
export OPENDUT_CLEO_NETWORK_CARL_HOST={carl_host}
export OPENDUT_CLEO_NETWORK_CARL_PORT={carl_port}
export OPENDUT_CLEO_NETWORK_OIDC_ENABLED={oidc_enabled}
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL={issuer_url}
export SSL_CERT_FILE=$CERT_PATH

exec ./{cleo_executable_name} "$@"
"#).to_string()
    }
}
