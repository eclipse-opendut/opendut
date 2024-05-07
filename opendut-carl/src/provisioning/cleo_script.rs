use config::Config;
use crate::util::CleoArch;

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

    pub fn build_script(&self, architecture: &CleoArch) -> String {
        format!(r#"#!/bin/bash

DIR_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
CERT_PATH=$DIR_PATH/{}

export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SCOPES=
export OPENDUT_CLEO_NETWORK_TLS_DOMAIN_NAME_OVERRIDE={}
export OPENDUT_CLEO_NETWORK_TLS_CA=$CERT_PATH
export OPENDUT_CLEO_NETWORK_CARL_HOST={}
export OPENDUT_CLEO_NETWORK_CARL_PORT={}
export OPENDUT_CLEO_NETWORK_OIDC_ENABLED={}
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL={}
export SSL_CERT_FILE=$CERT_PATH

exec ./{} "$@""#,
                crate::provisioning::cleo::CA_CERTIFICATE_FILE_NAME,
                self.carl_host,
                self.carl_host,
                self.carl_port,
                self.oidc_enabled,
                self.issuer_url,
                architecture.name()
        )
    }
}