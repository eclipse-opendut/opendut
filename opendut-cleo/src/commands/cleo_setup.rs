use std::path::Path;

use indoc::formatdoc;

use opendut_types::cleo::CleoSetup;
use opendut_types::util::net::AuthConfig;
use opendut_util::settings::SetupType;

use crate::{CleoSetupType, ParseableCleoSetupString};

/// CLEO setup for authenticating against CARL
#[derive(clap::Parser)]
pub struct CleoSetupCli {
    ///CLEO Setup string
    #[arg()]
    setup_string: ParseableCleoSetupString,
    ///Persist CLEO setup to file
    #[arg(value_enum, short, long, default_missing_value="user", num_args = 0..=1)]
    persistent: Option<CleoSetupType>,
}

impl CleoSetupCli {
    pub async fn execute(self) -> crate::Result<()> {
        let setup_string = *self.setup_string.0;
        
        match self.persistent {
            Some(persistence_type) => {
                let cleo_certificate_path = opendut_util::settings::write_certificate("cleo", setup_string.clone().ca.0, SetupType::from(persistence_type)).expect("Could not write certificate");
                let new_settings_string = prepare_cleo_configuration(setup_string, &cleo_certificate_path);
                opendut_util::settings::write_config("cleo", &new_settings_string, SetupType::User).unwrap();
                Ok(())
            }
            None => {
                let carl_host = setup_string.carl.host_str().expect("Host name should be defined in CARL URL.");
                let carl_port = setup_string.carl.port().unwrap_or(443);
                let ca_content = setup_string.ca.0.to_string();
                let mut environment_variables = formatdoc!{"
                OPENDUT_CLEO_NETWORK_TLS_DOMAIN_NAME_OVERRIDE={carl_host}
                OPENDUT_CLEO_NETWORK_TLS_CA_CONTENT=\"{ca_content}\"
                OPENDUT_CLEO_NETWORK_CARL_HOST={carl_host}
                OPENDUT_CLEO_NETWORK_CARL_PORT={carl_port}
            "};

                match setup_string.auth_config {
                    AuthConfig::Disabled => {
                        environment_variables.push_str(formatdoc! {"
                            OPENDUT_CLEO_NETWORK_OIDC_ENABLED=false
                        "}.as_str()
                        );
                    }
                    AuthConfig::Enabled { issuer_url, client_id, client_secret, .. } => {
                        let id = client_id.value();
                        let secret = client_secret.value();
                        environment_variables.push_str(formatdoc!{"
                            OPENDUT_CLEO_NETWORK_OIDC_ENABLED=true
                            OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL={issuer_url}
                            OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ID={id}
                            OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SECRET={secret}
                            OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SCOPES=\"\"
                        "}.as_str()
                        );
                    }
                }

                println!("{}", environment_variables);

                Ok(())
            }
        }
    }
}

fn prepare_cleo_configuration(setup_string: CleoSetup, cleo_ca_path: &Path) -> String {
    let mut new_settings = toml_edit::Document::new();

    let carl_host = setup_string.carl.host_str().expect("Host name should be defined in CARL URL.");
    let carl_port = setup_string.carl.port().unwrap_or(443);

    if new_settings.get("network").and_then(|network| network.get("carl")).is_none() {
        new_settings["network"] = toml_edit::table();
        new_settings["network"]["carl"] = toml_edit::table();
        new_settings["network"]["carl"].as_table_mut().unwrap().set_dotted(true);
    }
    new_settings["network"]["carl"]["host"] = toml_edit::value(carl_host);
    new_settings["network"]["carl"]["port"] = toml_edit::value(i64::from(carl_port));

    match setup_string.auth_config {
        AuthConfig::Disabled => {
            if new_settings.get("network").and_then(|network| network.get("oidc")).is_none() {
                new_settings["network"]["oidc"] = toml_edit::table();
            }
            new_settings["network"]["oidc"]["enabled"] = toml_edit::value(false);
        }
        AuthConfig::Enabled { client_id, client_secret, issuer_url, scopes} => {
            let network_oidc_client_id = client_id.clone().value();
            let network_oidc_client_secret = client_secret.clone().value();
            let network_oidc_client_issuer_url: String = issuer_url.clone().into();
            let network_oidc_client_scopes = scopes.clone().into_iter().map(|scope| scope.value()).collect::<Vec<_>>().join(",");

            if new_settings.get("network").and_then(|network| network.get("oidc")).is_none() {
                new_settings["network"]["oidc"] = toml_edit::table();
                new_settings["network"]["tls"] = toml_edit::table();
                new_settings["network"]["tls"]["domain"] = toml_edit::table();
                new_settings["network"]["tls"]["domain"].as_table_mut().unwrap().set_dotted(true);
                new_settings["network"]["tls"]["domain"]["name"] = toml_edit::table();
                new_settings["network"]["tls"]["domain"]["name"].as_table_mut().unwrap().set_dotted(true);
            }
            new_settings["network"]["oidc"]["enabled"] = toml_edit::value(true);
            new_settings["network"]["tls"]["ca"] = toml_edit::value(cleo_ca_path.to_str().unwrap());
            new_settings["network"]["tls"]["domain"]["name"]["override"]= toml_edit::value(carl_host);

            if new_settings.get("network")
                .and_then(|network| network.get("oidc"))
                .and_then(|network| network.get("client"))
                .is_none() {

                new_settings["network"]["oidc"]["client"] = toml_edit::table();
                new_settings["network"]["oidc"]["client"]["issuer"] = toml_edit::table();
                new_settings["network"]["oidc"]["client"]["issuer"].as_table_mut().unwrap().set_dotted(true);
            }
            new_settings["network"]["oidc"]["client"]["id"] = toml_edit::value(network_oidc_client_id);
            new_settings["network"]["oidc"]["client"]["secret"] = toml_edit::value(network_oidc_client_secret);
            new_settings["network"]["oidc"]["client"]["scopes"] = toml_edit::value(network_oidc_client_scopes);
            new_settings["network"]["oidc"]["client"]["issuer"]["url"] = toml_edit::value(network_oidc_client_issuer_url);
        }
    };

    new_settings.to_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::str::FromStr;
    use url::Url;
    use pem::Pem;

    use opendut_types::cleo::{CleoId, CleoSetup};
    use opendut_types::util::net::{AuthConfig, Certificate, ClientId, ClientSecret};
    use crate::commands::cleo_setup::prepare_cleo_configuration;

    #[test]
    fn prepare_cleo_configuration_with_auth_config_disabled(
    ) -> anyhow::Result<()> {
        let cleo_setup = CleoSetup {
            id: CleoId::random(),
            carl: Url::from_str("https://carl:1234/").unwrap(),
            ca: Certificate(Pem::new("Test Tag".to_string(), vec![])),
            auth_config: AuthConfig::Disabled,
        };

        let setup_string = prepare_cleo_configuration(cleo_setup, Path::new("/test/path/config.toml"));

        assert!(setup_string.contains("carl.host = \"carl\""));
        assert!(setup_string.contains("enabled = false"));

        Ok(())
    }

    #[test]
    fn prepare_cleo_configuration_with_auth_config_enabled(
    ) -> anyhow::Result<()> {
        let cleo_setup = CleoSetup {
            id: CleoId::random(),
            carl: Url::from_str("https://carl:1234/").unwrap(),
            ca: Certificate(Pem::new("Test Tag".to_string(), vec![])),
            auth_config: AuthConfig::Enabled {
                issuer_url: Url::from_str("https://auth:1234/").unwrap(),
                client_id: ClientId::from("testClient"),
                client_secret: ClientSecret::from("secret"),
                scopes: vec![],
            },
        };

        let setup_string = prepare_cleo_configuration(cleo_setup, Path::new("/test/path/config.toml"));

        assert!(setup_string.contains("carl.host = \"carl\""));
        assert!(setup_string.contains("enabled = true"));
        assert!(setup_string.contains("ca = \"/test/path/config.toml\""));
        assert!(setup_string.contains("issuer.url = \"https://auth:1234/\""));

        Ok(())
    }
}
