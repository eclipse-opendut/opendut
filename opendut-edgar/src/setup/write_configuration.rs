use crate::fs;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Context};
use config::Config;
use tracing::{debug, error, info};
use url::Url;

use opendut_model::peer::PeerId;
use opendut_model::util::net::AuthConfig;

use crate::common::settings;
use crate::common::settings::CONFIG_APPLICATION_PREFIX;
use crate::setup::constants;
use crate::setup::util::create_file_and_ensure_it_can_only_be_read_or_modified_by_owner;

#[derive(Clone)]
pub struct ConfigOverride {
    pub peer_id: PeerId,
    pub carl_url: Url,
    pub auth_config: AuthConfig,
}

pub fn write_with_override(config_override: ConfigOverride, no_confirm: bool) -> anyhow::Result<()> {
    write_with_options(WriteConfigurationOptions {
        config_file_to_write_to: settings::default_config_file_path(),
        config_merge_suggestion_file: constants::default_config_merge_suggestion_file_path(),
        config_override,
        no_confirm,
        user_attended: console::user_attended(),
    })
}


#[derive(Clone)]
struct WriteConfigurationOptions {
    config_file_to_write_to: PathBuf,
    config_merge_suggestion_file: PathBuf,
    config_override: ConfigOverride,
    no_confirm: bool,
    user_attended: bool,
}
fn write_with_options(options: WriteConfigurationOptions) -> anyhow::Result<()> {
    let WriteConfigurationOptions {
        config_file_to_write_to,
        config_merge_suggestion_file,
        config_override,
        no_confirm,
        user_attended,
    } = options;

    let original_settings = load_current_settings_from_file_and_env(&config_file_to_write_to)
        .unwrap_or_else(|| {
            debug!("Could not load settings from configuration file at {config_file_to_write_to:?}. Continuing as if no previous configuration exists.");
            toml_edit::DocumentMut::new()
        });

    let new_settings_string = update_settings(original_settings.clone(), &config_override).to_string();

    if !needs_writing_to_config_file(&new_settings_string, &config_file_to_write_to)? {
        debug!("The configuration on disk already matches the overrides we wanted to apply.");
        return Ok(())
    }

    let target_file_empty =
        config_file_to_write_to.exists().not()
        || config_file_to_write_to.metadata()?.len() == 0;


    let should_overwrite =
        if target_file_empty || no_confirm {
            true
        }
        else if user_attended {
            crate::setup::user_confirmation_prompt("Settings file already exists, but contains mismatched configurations! Do you want to overwrite it?")?
        }
        else {
            false
        };

    if should_overwrite {
        write_settings(&config_file_to_write_to, &new_settings_string)
            .context("Error while writing new configuration file.")?;

        info!("Successfully wrote peer configuration to: {config_file_to_write_to:?}");
        Ok(())
    } else {
        write_settings(&config_merge_suggestion_file, &new_settings_string)
            .context("Error while writing configuration merge suggestion file.")?;

        let message =
            String::from("Settings file already exists, but contains mismatched configurations!\n")
                + &format!("A suggestion for a merged configuration has been generated at {config_merge_suggestion_file:?}.\n")
                + &format!("Please check, if the configuration matches your expectation and if so, replace the configuration file at {config_file_to_write_to:?}.");
        Err(anyhow!(message))
    }
}

fn load_current_settings_from_file_and_env(path: &Path) -> Option<toml_edit::DocumentMut> {

    let config = opendut_util::settings::add_files_and_env_to_config_builder( //load from config file + env, so that envs specified during Setup are persisted for Service
        vec![path.to_path_buf()],
        CONFIG_APPLICATION_PREFIX,
        Config::builder()
    )
    .build()
    .inspect_err(|cause| error!("Failed to read existing configuration file at {path:?}. Continuing as if no configuration existed.\n  {cause}"))
    .ok()?;

    let current_settings = config.try_deserialize::<toml::Table>()
        .expect("config::Config should deserialize as toml::Table");

    match toml_edit::DocumentMut::from_str(&current_settings.to_string()) {
        Ok(current_settings) => Some(current_settings),
        Err(cause) => {
            error!("Failed to parse existing configuration as TOML.\n  {cause}");
            None
        }
    }
}

fn needs_writing_to_config_file(new_settings: &str, config_file: &Path) -> anyhow::Result<bool> {

    if config_file.exists().not() {
        debug!("Original config file does not exist. Assuming it needs to be written.");
        return Ok(true);
    }

    let current_settings = match fs::read_to_string(config_file) { //read anew from config file, because we do not want ENVs to be included here
        Ok(content) => content,
        Err(cause) => {
            error!("Failed to read existing configuration file at {config_file:?}. Will assume, it needs to be written.\n  {cause}");
            return Ok(true);
        }
    };
    let current_settings = toml::Table::from_str(&current_settings)?;

    let new_settings = toml::Table::from_str(new_settings)?; //cannot compare `toml_edit::DocumentMut` (no `PartialEq`) and comparing strings is error-prone due to re-ordered strings

    let result = (current_settings == new_settings).not();
    Ok(result)
}


fn update_settings(mut settings: toml_edit::DocumentMut, config_override: &ConfigOverride) -> toml_edit::DocumentMut {
    let ConfigOverride { peer_id, carl_url, auth_config } = config_override;

    let peer_id = peer_id.to_string();
    let carl_host = carl_url.host_str().expect("Host name should be defined in CARL URL.");
    let carl_port = carl_url.port().unwrap_or(443);

    if settings.get("peer").is_none() {
        settings["peer"] = toml_edit::table();
    }
    settings["peer"]["id"] = toml_edit::value(peer_id);

    if settings.get("network").is_none() {
        settings["network"] = toml_edit::table();
    }
    if settings["network"].get("carl").is_none() {
        settings["network"]["carl"] = toml_edit::table();
        settings["network"]["carl"].as_table_mut().unwrap().set_dotted(true);
    }
    settings["network"]["carl"]["host"] = toml_edit::value(carl_host);
    settings["network"]["carl"]["port"] = toml_edit::value(i64::from(carl_port));

    match &auth_config {
        AuthConfig::Disabled => {
            if settings["network"].get("oidc").is_none() {
                settings["network"]["oidc"] = toml_edit::table();
            }
            settings["network"]["oidc"]["enabled"] = toml_edit::value(false);
        }
        AuthConfig::Enabled { client_id, client_secret, issuer_url, scopes } => {
            let network_oidc_client_id = client_id.clone().value();
            let network_oidc_client_secret = client_secret.clone().value();
            let network_oidc_client_scopes = scopes.clone().into_iter().map(|scope| scope.value()).collect::<Vec<_>>().join(",");
            let network_oidc_client_issuer_url: String = issuer_url.clone().into();

            if settings["network"].get("oidc").is_none() {
                settings["network"]["oidc"] = toml_edit::table();
            }
            settings["network"]["oidc"]["enabled"] = toml_edit::value(true);

            if settings["network"]["oidc"].get("client").is_none() {
                settings["network"]["oidc"]["client"] = toml_edit::table();
                settings["network"]["oidc"]["client"]["issuer"] = toml_edit::table();
                settings["network"]["oidc"]["client"]["issuer"].as_table_mut().unwrap().set_dotted(true);
            }
            settings["network"]["oidc"]["client"]["id"] = toml_edit::value(network_oidc_client_id);
            settings["network"]["oidc"]["client"]["secret"] = toml_edit::value(network_oidc_client_secret);
            settings["network"]["oidc"]["client"]["scopes"] = toml_edit::value(network_oidc_client_scopes);
            settings["network"]["oidc"]["client"]["issuer"]["url"] = toml_edit::value(network_oidc_client_issuer_url);
        }
    };

    settings
}


fn write_settings(target: &Path, settings_string: &str) -> anyhow::Result<()> {
    let parent_dir = target
        .parent()
        .ok_or(anyhow!("Expected configuration file {target:?} to have a parent directory."))?;
    fs::create_dir_all(parent_dir)?;
    create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(target)?;

    fs::write(target, settings_string)
        .context(format!("Error while writing to configuration file at {target:?}."))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::MutexGuard;
    use assert_fs::fixture::ChildPath;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use indoc::{formatdoc, indoc};
    use predicates::boolean::PredicateBooleanExt;
    use predicates::Predicate;
    use predicates::prelude::predicate;
    use toml::toml;
    use uuid::uuid;
    use googletest::prelude::*;
    use opendut_model::util::net::{ClientId, ClientSecret, OAuthScope};

    use super::*;

    const HOST: &str = "example.com";
    const PORT: u16 = 1234;
    const CLIENT_ID: &str = "ClientId";
    const CLIENT_SECRET: &str = "ClientSecret";
    const OIDC_ENABLED: bool = true;
    const ISSUER_URL: &str = "https://test.com:1234/";
    const SCOPES: &str = "test";


    #[test]
    fn should_write_a_fresh_configuration_with_auth_config_enabled() -> anyhow::Result<()> {
        let fixture = Fixture::new();
        let options = create_write_configuration_options(&fixture, AuthEnabled::Yes);

        assert!(predicate::path::missing().eval(&options.config_file_to_write_to));

        let path = options.config_file_to_write_to.clone();

        write_with_options(options)?;

        assert!(predicate::path::exists().eval(&path));
        let file_content = fs::read_to_string(&path)?;

        assert_that!(file_content, eq(indoc!(r#"
            [peer]
            id = "dc72f6d9-d700-455f-8c31-9f15438e7503"

            [network]
            carl.host = "example.com"
            carl.port = 1234

            [network.oidc]
            enabled = true

            [network.oidc.client]
            issuer.url = "https://test.com:1234/"
            id = "ClientId"
            secret = "ClientSecret"
            scopes = "test"
        "#)));

        Ok(())
    }

    #[test]
    fn should_write_a_fresh_configuration_with_auth_config_disabled() -> anyhow::Result<()> {
        let fixture = Fixture::new();
        let options = create_write_configuration_options(&fixture, AuthEnabled::No);

        let path = options.config_file_to_write_to.clone();

        assert!(predicate::path::missing().eval(&path));

        write_with_options(options)?;

        assert!(predicate::path::exists().eval(&path));
        let file_content = fs::read_to_string(&path)?;

        assert_that!(file_content, eq(indoc!(r#"
            [peer]
            id = "dc72f6d9-d700-455f-8c31-9f15438e7503"

            [network]
            carl.host = "example.com"
            carl.port = 1234

            [network.oidc]
            enabled = false
        "#)));

        Ok(())
    }

    #[test]
    fn should_provide_a_merge_suggestion_for_an_already_existing_configuration_but_should_not_delete_existing_unknown_keys() -> anyhow::Result<()> {
        let fixture = Fixture::new();
        let options = create_write_configuration_options(&fixture, AuthEnabled::Yes);

        let config_file = ChildPath::new(options.config_file_to_write_to.clone());
        let config_merge_suggestion_file = ChildPath::new(options.config_merge_suggestion_file.clone());

        config_file.write_str(indoc!(r#"
            [peer]
            id = "eef8997e-56a0-4d8d-978e-40d1f2e68db0"
            [peer.unknown]
            key = "value"
            [Hallo.Welt]
            key = "value"
        "#))?;

        let file_content = fs::read_to_string(&config_file)?;
        assert!(predicate::str::is_empty().not().eval(&file_content));
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        let result = write_with_options(options);
        assert!(result.is_err());

        assert!(predicate::path::exists().eval(&config_merge_suggestion_file));
        let merge_suggestion = fs::read_to_string(config_merge_suggestion_file)?;
        assert!(predicate::str::contains(fixture.peer_id.to_string()).eval(&merge_suggestion));
        assert!(predicate::str::contains("[peer.unknown]".to_string()).eval(&merge_suggestion));
        assert!(predicate::str::contains("[Hallo.Welt]".to_string()).eval(&merge_suggestion));

        Ok(())
    }

    #[test]
    fn should_provide_a_merge_suggestion_for_an_already_existing_configuration_with_auth_config_disabled() -> anyhow::Result<()> {
        let fixture = Fixture::new();
        let options = create_write_configuration_options(&fixture, AuthEnabled::No);

        let config_file = ChildPath::new(options.config_file_to_write_to.clone());
        let config_merge_suggestion_file = ChildPath::new(options.config_merge_suggestion_file.clone());

        config_file.write_str(&formatdoc!(r#"
            [peer]
            id = "eef8997e-56a0-4d8d-978e-40d1f2e68db0"

            [network.oidc]
            enabled = {OIDC_ENABLED}

            [network.oidc.client]
            id = "{CLIENT_ID}"
            secret = "{CLIENT_SECRET}"
        "#))?;

        let file_content = fs::read_to_string(&config_file)?;
        assert!(predicate::str::is_empty().not().eval(&file_content));
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        let result = write_with_options(options);
        assert!(result.is_err());

        assert!(predicate::path::exists().eval(&config_merge_suggestion_file));
        let merge_suggestion = fs::read_to_string(config_merge_suggestion_file)?;
        assert!(predicate::str::contains(fixture.peer_id.to_string()).eval(&merge_suggestion));
        assert!(predicate::str::contains("enabled = false".to_string()).eval(&merge_suggestion));

        Ok(())
    }

    #[test]
    fn should_not_provide_a_merge_suggestion_if_the_existing_config_matches() -> anyhow::Result<()> {
        let fixture = Fixture::new();
        let options = create_write_configuration_options(&fixture, AuthEnabled::Yes);

        let config_file = ChildPath::new(options.config_file_to_write_to.clone());
        let config_merge_suggestion_file = ChildPath::new(options.config_merge_suggestion_file.clone());

        config_file.write_str(&formatdoc!(r#"
            [peer]
            id = "{peer_id}"

            [network.carl]
            host = "{HOST}"
            port = {PORT}

            [network.oidc]
            enabled = {OIDC_ENABLED}

            [network.oidc.client]
            issuer.url = "{ISSUER_URL}"
            id = "{CLIENT_ID}"
            secret = "{CLIENT_SECRET}"
            scopes = "{SCOPES}"
        "#, peer_id=fixture.peer_id))?;

        let file_content = fs::read_to_string(&config_file)?;
        assert!(predicate::str::is_empty().not().eval(&file_content));
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        let result = write_with_options(options);
        assert!(result.is_ok());
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        Ok(())
    }

    mod env {
        use super::*;

        #[test]
        fn should_persist_config_values_configured_via_envs_into_empty_file() -> anyhow::Result<()> {
            let fixture = Fixture::new();
            let options = create_write_configuration_options(&fixture, AuthEnabled::No);

            let path = options.config_file_to_write_to.clone();

            let env_value = with_env(&||
                write_with_options(options.clone())
            )?;

            let file_content = fs::read_to_string(&path)?;
            assert!(file_content.contains(env_value));
            Ok(())
        }

        /// The envs should not be included into the existing settings when comparing with the new settings (which include the envs),
        /// while determining whether the configuration file should be written, as we do want to persist these environment variable values
        /// and they should therefore appear in the configuration file, going forward.
        #[test]
        fn should_take_envs_into_consideration_for_determining_when_an_override_is_needed() -> anyhow::Result<()> {
            let fixture = Fixture::new();
            let options = create_write_configuration_options(&fixture, AuthEnabled::No);

            write_with_options(options.clone())?; //pre-create to force override


            let result = with_env(&||
                write_with_options(options.clone())
            );


            assert!(result.is_err());
            assert!(predicate::path::exists().eval(&options.config_merge_suggestion_file));
            Ok(())
        }
    }

    #[test]
    fn should_determine_whether_the_configuration_needs_writing() -> anyhow::Result<()> {
        let file = assert_fs::NamedTempFile::new("opendut-test-should_determine_whether_the_configuration_needs_writing.txt")?;

        let new_settings = toml! {
            [peer]
            id = "ffffffff-048b-4c5b-9a2f-43bf1d6d5a03"
        };

        let needed = needs_writing_to_config_file(&new_settings.to_string(), file.path())?;
        assert!(needed, "Should need to be written, when no file exists.");


        let current_settings = toml! {
            [peer]
            id = "00000000-fe18-4a4c-aa9b-4ef15e59cbdc"
        };
        file.write_str(&current_settings.to_string())?;

        let needed = needs_writing_to_config_file(&new_settings.to_string(), file.path())?;
        assert!(needed, "Should need to be written, when the new settings differ from the current settings.");


        let new_settings = current_settings.clone();
        let needed = needs_writing_to_config_file(&new_settings.to_string(), file.path())?;
        assert!(needed.not(), "Should not need to be written, when the new and current settings are equivalent.");

        Ok(())
    }




    fn create_write_configuration_options(
        fixture: &Fixture,
        auth_enabled: AuthEnabled,
    ) -> WriteConfigurationOptions {

        let config_override = match auth_enabled {
            AuthEnabled::Yes => ConfigOverride {
                peer_id: fixture.peer_id,
                carl_url: Url::parse("https://example.com:1234").unwrap(),
                auth_config: AuthConfig::Enabled {
                    issuer_url: Url::parse("https://test.com:1234").unwrap(),
                    client_secret: ClientSecret::from(CLIENT_SECRET),
                    client_id: ClientId::from(CLIENT_ID),
                    scopes: vec![OAuthScope("test".to_string())],
                },
            },
            AuthEnabled::No => ConfigOverride {
                peer_id: fixture.peer_id,
                carl_url: Url::parse("https://example.com:1234").unwrap(),
                auth_config: AuthConfig::Disabled,
            },
        };

        WriteConfigurationOptions {
            config_file_to_write_to: fixture.config_file_to_write_to.to_path_buf(),
            config_merge_suggestion_file: fixture.config_merge_suggestion_file.to_path_buf(),
            config_override,
            no_confirm: false,
            user_attended: false, // prevent hanging in unit tests
        }
    }
    enum AuthEnabled { Yes, No }



    struct Fixture<'a> {
        _temp_dir: TempDir,
        config_file_to_write_to: ChildPath,
        config_merge_suggestion_file: ChildPath,
        peer_id: PeerId,
        /// Blocks other tests from reading the envs while the given test is running.
        /// Otherwise, the modified env will influence other tests running in parallel.
        _env_access_guard: MutexGuard<'a, ()>, //carry along to drop guard at the end of the test
    }
    impl Fixture<'_> {

        fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();

            let config_file_to_write_to = temp_dir.child("edgar.toml");

            let config_merge_suggestion_file = temp_dir.child("edgar-merge-suggestion.toml");

            let peer_id = PeerId::from(uuid!("dc72f6d9-d700-455f-8c31-9f15438e7503"));

            let _env_access_guard = ENV_ACCESS.lock().unwrap();

            Self {
                _temp_dir: temp_dir,
                config_file_to_write_to,
                config_merge_suggestion_file,
                peer_id,
                _env_access_guard,
            }
        }
    }

    static ENV_ACCESS: Mutex<()> = Mutex::new(()); //decided against using `RwLock`, because we couldn't hand out the `RwLock{Read,Write}Guard` with the Fixture, without the lifetime being dropped immediately and therefore the tests not blocking each other after all. (Tried sticking them in `Box<dyn Any>` and in an enum.)

    fn with_env(code: &impl Fn() -> anyhow::Result<()>) -> anyhow::Result<&'static str> {
        let env_key = "OPENDUT_EDGAR_NETWORK_CONNECT_RETRIES";
        let env_value = "123456789";

        unsafe { std::env::set_var(env_key, env_value); }

        let result = code();

        unsafe { std::env::remove_var(env_key); }
        result
            .map(|_| env_value)
    }
}
