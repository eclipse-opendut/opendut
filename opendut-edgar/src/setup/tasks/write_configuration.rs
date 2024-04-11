use std::fs;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Context};
use tracing::{debug, error};
use url::Url;

use opendut_types::peer::PeerId;

use crate::common::settings;
use crate::setup::constants;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct ConfigOverride {
    pub peer_id: PeerId,
    pub carl_url: Url,
}

pub struct WriteConfiguration {
    config_file_to_write_to: PathBuf,
    config_merge_suggestion_file: PathBuf,
    config_override: ConfigOverride,
}

impl Task for WriteConfiguration {
    fn description(&self) -> String {
        String::from("Write Configuration")
    }
    fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> anyhow::Result<Success> {
        let original_settings = self.load_current_settings()
            .unwrap_or_else(|| {
                debug!("Could not load settings from configuration file at '{}'. Continuing as if no previous configuration exists.", self.config_file_to_write_to.display());
                toml_edit::Document::new()
            });

        let new_settings_string = {
            let mut new_settings = Clone::clone(&original_settings);

            let peer_id = self.config_override.peer_id.to_string();
            let carl_host = self.config_override.carl_url.host_str().expect("Host name should be defined in CARL URL.");
            let carl_port = self.config_override.carl_url.port().unwrap_or(443);

            if new_settings.get("peer").is_none() {
                new_settings["peer"] = toml_edit::table();
            }
            new_settings["peer"]["id"] = toml_edit::value(peer_id);

            if new_settings.get("network").and_then(|network| network.get("carl")).is_none() {
                new_settings["network"] = toml_edit::table();
                new_settings["network"]["carl"] = toml_edit::table();
                new_settings["network"]["carl"].as_table_mut().unwrap().set_dotted(true);
            }
            new_settings["network"]["carl"]["host"] = toml_edit::value(carl_host);
            new_settings["network"]["carl"]["port"] = toml_edit::value(i64::from(carl_port));

            new_settings.to_string()
        };

        if original_settings.to_string() == new_settings_string {
            debug!("The configuration on disk already matches the overrides we wanted to apply.");
            return Ok(Success::message("Configuration on disk matches."))
        }

        if self.config_file_to_write_to.exists()
        && self.config_file_to_write_to.metadata()?.len() > 0 {
            write_settings(&self.config_merge_suggestion_file, &new_settings_string)
                .context("Error while writing configuration merge suggestion file.")?;
            let message =
                String::from("Settings file already exists, but does not contain necessary configurations!\n")
                + &format!("A suggestion for a merged configuration has been generated at '{}'.\n", self.config_merge_suggestion_file.display())
                + &format!("Please check, if the configuration matches your expectation and if so, replace the configuration file at '{}'.", self.config_file_to_write_to.display());
            Err(anyhow!(message))
        } else {
            write_settings(&self.config_file_to_write_to, &new_settings_string)
                .context("Error while writing new configuration file.")?;
            Ok(Success::default())
        }
    }
}
impl WriteConfiguration {
    pub fn with_override(config_override: ConfigOverride) -> Self {
        Self {
            config_file_to_write_to: settings::default_config_file_path(),
            config_merge_suggestion_file: constants::default_config_merge_suggestion_file_path(),
            config_override,
        }
    }

    fn load_current_settings(&self) -> Option<toml_edit::Document> {

        if self.config_file_to_write_to.exists().not() {
            return None;
        }

        let current_settings = match fs::read_to_string(&self.config_file_to_write_to) {
            Ok(content) => content,
            Err(cause) => {
                error!("Failed to read existing configuration file at '{}'.\n  {cause}", self.config_file_to_write_to.display());
                return None;
            }
        };

        match toml_edit::Document::from_str(&current_settings) {
            Ok(current_settings) => Some(current_settings),
            Err(cause) => {
                error!("Failed to parse existing configuration as TOML.\n  {cause}");
                None
            }
        }
    }
}

fn write_settings(target: &Path, settings_string: &str) -> anyhow::Result<()> {
    let parent_dir = target
        .parent()
        .ok_or(anyhow!("Expected configuration file '{}' to have a parent directory.", target.display()))?;
    fs::create_dir_all(parent_dir)?;

    fs::write(target, settings_string)
        .context(format!("Error while writing to configuration file at '{}'.", target.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use indoc::indoc;
    use predicates::boolean::PredicateBooleanExt;
    use predicates::Predicate;
    use predicates::prelude::predicate;
    use uuid::uuid;
    use googletest::prelude::*;

    use crate::setup::runner;

    use super::*;

    const HOST: &str = "example.com";
    const PORT: u16 = 1234;

    #[test]
    fn should_write_a_fresh_configuration() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let config_file = temp.child("edgar.toml");
        let config_merge_suggestion_file = temp.child("edgar-merge-suggestion.toml");

        let new_peer_id = PeerId::from(uuid!("dc72f6d9-d700-455f-8c31-9f15438e7503"));

        let task = WriteConfiguration {
            config_file_to_write_to: config_file.to_path_buf(),
            config_merge_suggestion_file: config_merge_suggestion_file.to_path_buf(),
            config_override: ConfigOverride {
                peer_id: new_peer_id,
                carl_url: Url::parse("https://example.com:1234").unwrap(),
            },
        };

        assert!(predicate::path::missing().eval(&config_file));

        runner::test::unchecked(task)?;

        assert!(predicate::path::exists().eval(&config_file));
        let file_content = fs::read_to_string(&config_file)?;

        assert_that!(file_content, eq(indoc!(r#"
            [peer]
            id = "dc72f6d9-d700-455f-8c31-9f15438e7503"

            [network]
            carl.host = "example.com"
            carl.port = 1234
        "#)));

        Ok(())
    }

    #[test]
    fn should_provide_an_merge_suggestion_for_an_already_existing_configuration() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let config_file = temp.child("edgar.toml");
        let config_merge_suggestion_file = temp.child("edgar-merge-suggestion.toml");

        config_file.write_str(indoc!(r#"
            [peer]
            id = "eef8997e-56a0-4d8d-978e-40d1f2e68db0"
        "#))?;

        let new_peer_id = PeerId::random();

        let task = WriteConfiguration {
            config_file_to_write_to: config_file.to_path_buf(),
            config_merge_suggestion_file: config_merge_suggestion_file.to_path_buf(),
            config_override: ConfigOverride {
                peer_id: new_peer_id,
                carl_url: Url::parse("https://example.com:1234").unwrap(),
            },
        };

        let file_content = fs::read_to_string(&config_file)?;
        assert!(predicate::str::is_empty().not().eval(&file_content));
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        let result = runner::test::unchecked(task);
        assert!(result.is_err());

        assert!(predicate::path::exists().eval(&config_merge_suggestion_file));
        let merge_suggestion = fs::read_to_string(config_merge_suggestion_file)?;
        assert!(predicate::str::contains(new_peer_id.to_string()).eval(&merge_suggestion));

        Ok(())
    }

    #[test]
    fn should_not_provide_a_merge_suggestion_if_the_existing_config_matches() -> anyhow::Result<()> {
        let temp = TempDir::new()?;
        let config_file = temp.child("edgar.toml");
        let config_merge_suggestion_file = temp.child("edgar-merge-suggestion.toml");

        let new_peer_id = PeerId::random();

        config_file.write_str(&format!(indoc!(r#"
            [peer]
            id = "{}"

            [network.carl]
            host = "{}"
            port = {}
        "#), new_peer_id, HOST, PORT))?;

        let task = WriteConfiguration {
            config_file_to_write_to: config_file.to_path_buf(),
            config_merge_suggestion_file: config_merge_suggestion_file.to_path_buf(),
            config_override: ConfigOverride {
                peer_id: new_peer_id,
                carl_url: Url::parse(&format!("https://{HOST}:{PORT}")).unwrap(),
            },
        };

        let file_content = fs::read_to_string(&config_file)?;
        assert!(predicate::str::is_empty().not().eval(&file_content));
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        let result = runner::test::unchecked(task);
        assert!(result.is_ok());
        assert!(predicate::path::missing().eval(&config_merge_suggestion_file));

        Ok(())
    }
}
