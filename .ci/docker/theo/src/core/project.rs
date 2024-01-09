use std::collections::HashMap;
use std::env;
use std::env::VarError;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use std::io::Write;
use strum::{Display, EnumString, EnumVariantNames, VariantNames};

use crate::core::metadata::cargo_netbird_versions;
use crate::core::util::consume_output;

#[derive(Debug, PartialEq, EnumString, EnumVariantNames, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TheoDynamicEnvVars {
    Puser,
    Pgroup,
    Puid,
    Pgid,
    DockerUser,
    DockerGid,
    OpendutRepoRoot,
    NetbirdSignalVersion,
    NetbirdManagementVersion,
    NetbirdDashboardVersion,
    OpendutCarlVersion,
    OpendutCustomCa1,
    OpendutCustomCa2,
    OpendutHosts,
}

#[derive(Debug, PartialEq, EnumString, EnumVariantNames, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TheoUserEnvVars {
    OpendutDockerImageNamespace,
    OpendutDockerImageHost,
}

#[derive(Debug, Clone)]
pub struct TheoUserEnvMap(HashMap<String, String>);

impl Default for TheoUserEnvMap {
    fn default() -> Self {
        let mut env_map = HashMap::new();
        env_map.insert(TheoUserEnvVars::OpendutDockerImageNamespace.to_string(), "docker-opendut-namespace".to_string());
        env_map.insert(TheoUserEnvVars::OpendutDockerImageHost.to_string(), "docker-registry.example.com".to_string());
        Self(env_map)
    }
}

impl From<TheoUserEnvMap> for String {
    fn from(value: TheoUserEnvMap) -> Self {
        let mut result = value.0.into_iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>();
        result.sort();
        result.join("\n")
    }
}

pub trait ProjectRootDir {
    fn project_dir() -> String;
    fn project_path_buf() -> PathBuf;
}

impl ProjectRootDir for PathBuf {
    fn project_dir() -> String {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output();
        consume_output(output).expect("Failed to determine git project root directory")
    }

    fn project_path_buf() -> PathBuf {
        PathBuf::from(PathBuf::project_dir())
    }
}


fn get_docker_group_id() -> String {
    let docker_getent_group = consume_output(Command::new("getent").arg("group").arg("docker").output()).expect("Failed to get docker group.");
    let docker_group_id = docker_getent_group.split(":").nth(2).expect("Failed to get docker group id").to_string();
    docker_group_id
}

#[derive(Debug, Clone)]
pub struct TheoDynamicEnvMap(HashMap<String, String>);

impl Default for TheoDynamicEnvMap {
    fn default() -> Self {
        let user_id = consume_output(Command::new("id").arg("--user").output()).expect("Failed to get user id");

        let group_name = match consume_output(Command::new("id").arg("--group").arg("--name").output()) {
            Ok(group_name) => { group_name }
            Err(error) => {
                println!("Failed to get group name: {:?}. Using 'general' as fallback name for your group.", error);
                "general".to_string()
            }
        };
        let group_id = consume_output(Command::new("id").arg("--group").output()).expect("Failed to get group id");
        let docker_gid = get_docker_group_id();
        let git_repo_root = PathBuf::project_dir();
        let docker_user = format!("{}:{}", user_id.clone(), group_id.clone());

        let metadata = cargo_netbird_versions();

        fn read_pem_certificate() -> String {
            let ca_pem_file = PathBuf::project_path_buf().join("resources").join("development").join("tls").join("insecure-development-ca.pem");
            let pem_file = ca_pem_file.to_str().unwrap();
            let pem_file = std::fs::read_to_string(pem_file).expect("Failed to insecure development ca pem file.").replace("\n", "\\n").trim_end().to_string();
            pem_file
        }

        let mut env_map = HashMap::new();
        env_map.insert(TheoDynamicEnvVars::Puser.to_string(),
                       consume_output(Command::new("id").arg("--user").arg("--name").output()).expect("Failed to get user name"));
        env_map.insert(TheoDynamicEnvVars::Puid.to_string(),
                       user_id.clone());
        env_map.insert(TheoDynamicEnvVars::Pgroup.to_string(), group_name);
        env_map.insert(TheoDynamicEnvVars::Pgid.to_string(), group_id.clone());
        env_map.insert(TheoDynamicEnvVars::DockerUser.to_string(), docker_user.clone());
        env_map.insert(TheoDynamicEnvVars::DockerGid.to_string(), docker_gid.clone());
        env_map.insert(TheoDynamicEnvVars::OpendutRepoRoot.to_string(), git_repo_root.clone());
        env_map.insert(TheoDynamicEnvVars::NetbirdManagementVersion.to_string(), metadata.netbird.netbird_management_version.clone());
        env_map.insert(TheoDynamicEnvVars::NetbirdSignalVersion.to_string(), metadata.netbird.netbird_signal_version.clone());
        env_map.insert(TheoDynamicEnvVars::NetbirdDashboardVersion.to_string(), metadata.netbird.netbird_dashboard_version.clone());
        env_map.insert(TheoDynamicEnvVars::OpendutCarlVersion.to_string(), metadata.carl_version.clone());
        env_map.insert(TheoDynamicEnvVars::OpendutCustomCa1.to_string(), format!("\"{}\"", read_pem_certificate()));
        env_map.insert(TheoDynamicEnvVars::OpendutCustomCa2.to_string(), format!("\"{}\"", read_pem_certificate()));
        env_map.insert(TheoDynamicEnvVars::OpendutHosts.to_string(), "".to_string());

        Self(env_map)
    }
}

impl From<TheoDynamicEnvMap> for String {
    fn from(value: TheoDynamicEnvMap) -> Self {
        let mut result = value.0.into_iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>();
        result.sort();
        result.join("\n")
    }
}

pub(crate) fn write_theo_dynamic_env_vars() {
    let env_map = TheoDynamicEnvMap::default();
    let env_map_string: String = String::from(env_map.clone());
    let mut env_file = PathBuf::project_path_buf();
    env_file.push(".env-theo");
    std::fs::write(env_file, format!("{}\n", env_map_string)).expect("Failed to write .env-theo file.");
}

pub(crate) fn check_user_provided_dot_env_variables() {
    let env_map = TheoUserEnvMap::default();
    let env_file = PathBuf::project_path_buf().join(".env");
    if !env_file.exists() {
        std::fs::write(env_file.clone(), "").expect("Failed to write .env file.");
    }

    let mut missing_env_vars = false;
    for (env_key, env_value) in env_map.0.iter() {
        match env::var(env_key) {
            Ok(_) => { }
            Err(error) => {
                match error {
                    VarError::NotPresent => {
                        missing_env_vars = true;
                        println!("Environment variable '{}' is not set. Using a default value '{}'.", env_key, env_value);
                        let mut file = OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(env_file.clone())
                            .unwrap();

                        if let Err(e) = writeln!(file, "{}={}", env_key, env_value) {
                            eprintln!("Couldn't write to '.env' file: {}", e);
                        }
                    }
                    VarError::NotUnicode(_) => {
                        panic!("Environment variable '{}' is not unicode.", env_key);
                    }
                }
            }
        }
    }
    if missing_env_vars {
        println!("Some environment variables were not set. Default values were used. Please check/update them in the .env file.");
    }

}

pub(crate) fn check_dot_env_variables() {
    let mut missing_env_vars = "".to_owned();
    let mut error_messages = "".to_owned();

    let env_map = TheoDynamicEnvMap::default();

    for (env_key, env_value) in env_map.0.iter() {
        match env::var(env_key) {
            Ok(value) => {
                let value_without_newlines = value.clone().replace("\n", "\\n");
                let env_value = env_value.clone().trim_matches('"').to_string();
                // check if all environment variables are set correctly
                if value_without_newlines != *env_value {
                    let wrong_key_value = format!("Env variable is set as '{}={}'. Expected: '{}={}'\n", env_key, value_without_newlines, env_key, env_value);
                    println!("{}", wrong_key_value);
                    error_messages.push_str(&wrong_key_value);
                }
            }
            Err(_) => {
                // check if all required environment variables are set
                let missing_key_value = format!("{}={}\n", env_key, env_value);
                missing_env_vars.push_str(&missing_key_value);
            }
        };
    }

    if !missing_env_vars.is_empty() {
        println!("Missing environment variables in file '.env': \n{}", missing_env_vars);
    }
    if missing_env_vars.len() > 0 || error_messages.len() > 0 {
        panic!("There are errors in the environment variables in file '.env': \n{}", error_messages);
    }

    assert_eq!(["PUSER", "PGROUP", "PUID", "PGID", "DOCKER_USER", "DOCKER_GID", "OPENDUT_REPO_ROOT", "NETBIRD_SIGNAL_VERSION", "NETBIRD_MANAGEMENT_VERSION", "NETBIRD_DASHBOARD_VERSION",
                   "OPENDUT_CARL_VERSION", "OPENDUT_CUSTOM_CA1", "OPENDUT_CUSTOM_CA2", "OPENDUT_HOSTS"], TheoDynamicEnvVars::VARIANTS);
}


pub(crate) fn boolean_env_var(name: &str) -> bool {
    env::var(name).unwrap_or("false".to_string()) == "true".to_string()
}