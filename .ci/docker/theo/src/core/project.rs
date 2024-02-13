use std::collections::HashMap;
use std::env;
use std::env::VarError;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use dotenvy::dotenv;
use strum::{Display, EnumString, EnumVariantNames};

use crate::commands::vagrant::running_in_opendut_vm;
use crate::core::{OPENDUT_REPO_ROOT, OPENDUT_VM_NAME, TheoError};
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
    OpendutEdgarReplicas,
    OpendutEdgarClusterName,
    OpendutExposePorts,
}

#[derive(Debug, PartialEq, EnumString, EnumVariantNames, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TheoUserEnvVars {
    OpendutDockerImageNamespace,
    OpendutDockerImageHost,
}

#[derive(Debug, Clone)]
pub struct TheoEnvMap(HashMap<String, String>);

impl TheoEnvMap {
    fn user_default() -> Self {
        let mut env_map = HashMap::new();
        env_map.insert(TheoUserEnvVars::OpendutDockerImageNamespace.to_string(), "docker-opendut-namespace".to_string());
        env_map.insert(TheoUserEnvVars::OpendutDockerImageHost.to_string(), "docker-registry.example.com".to_string());
        Self(env_map)
    }

    fn theo_default() -> Self {
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
            let pem_file = std::fs::read_to_string(pem_file).expect("Failed to insecure development ca pem file.").replace('\n', "\\n").trim_end().to_string();
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
        env_map.insert(TheoDynamicEnvVars::OpendutEdgarReplicas.to_string(), "4".to_string());

        let cluster_suffix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("Failed to get time since epoch").as_secs().to_string();
        env_map.insert(TheoDynamicEnvVars::OpendutEdgarClusterName.to_string(), format!("cluster{}", cluster_suffix));
        if running_in_opendut_vm() {
            println!("Running in virtual machine '{}'. Automatically exposing ports within the virtual machine!", OPENDUT_VM_NAME);
            env_map.insert(TheoDynamicEnvVars::OpendutExposePorts.to_string(), "true".to_string());
        } else {
            env_map.insert(TheoDynamicEnvVars::OpendutExposePorts.to_string(), "false".to_string());
        }

        Self(env_map)
    }
}

impl From<TheoEnvMap> for String {
    fn from(value: TheoEnvMap) -> Self {
        let mut result = value.0.into_iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>();
        result.sort();
        result.join("\n")
    }
}

pub trait ProjectRootDir {
    fn project_dir() -> String;
    fn project_dir_verify();
    fn project_path_buf() -> PathBuf;
}

fn git_repo_root() -> Result<String, TheoError> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();
    consume_output(output)
}

impl ProjectRootDir for PathBuf {
    fn project_dir() -> String {
        env::var(OPENDUT_REPO_ROOT)
            .unwrap_or_else(|_| git_repo_root().expect("Failed to determine git project root directory"))
    }

    fn project_dir_verify() {
        let repo_root_env = env::var(OPENDUT_REPO_ROOT);
        let project_dir = match repo_root_env {
            Ok(repo_root) => {
                PathBuf::from(repo_root)
            }
            Err(_) => {
                PathBuf::from(git_repo_root().expect("Failed to determine git project root directory. You may use the 'OPENDUT_REPO_ROOT' environment variable instead."))
            }
        };
        if !project_dir.exists() {
            panic!("Could not determine project root directory. Check if you are in the correct directory or set environment variable OPENDUT_REPO_ROOT.");
        }
        if !project_dir.join("Cargo.toml").exists() {
            panic!("Could not find 'Cargo.toml'. Check if you are in the correct directory or set environment variable OPENDUT_REPO_ROOT.");
        }
    }

    fn project_path_buf() -> PathBuf {
        PathBuf::from(PathBuf::project_dir())
    }
}


fn get_docker_group_id() -> String {
    let docker_getent_group = consume_output(Command::new("getent").arg("group").arg("docker").output()).expect("Failed to get docker group.");
    let docker_group_id = docker_getent_group.split(':').nth(2).expect("Failed to get docker group id").to_string();
    docker_group_id
}


pub(crate) fn dot_env_create_theo_specific_defaults() {
    let theo_env_defaults = TheoEnvMap::theo_default();

    let mut env_map = HashMap::new();
    for (default_env_key, default_env_value) in theo_env_defaults.0.iter() {
        match env::var(default_env_key) {
            Ok(_existing_env_value) => { /* do not override existing values */ }
            Err(_error) => {
                env_map.insert(default_env_key.clone(), default_env_value.to_string());
            }
        }
    }

    let env_map = TheoEnvMap(env_map);
    let env_map_string: String = String::from(env_map.clone());
    let theo_env_file = PathBuf::project_path_buf().join(".env-theo");

    std::fs::write(theo_env_file, format!("{}\n", env_map_string)).expect("Failed to write .env-theo file.");
}

pub(crate) fn load_theo_environment_variables() {
    dot_env_create_theo_specific_defaults();
    let custom_env = PathBuf::project_path_buf().join(".env-theo");
    dotenvy::from_path(custom_env).expect(".env-theo file not found");
}

pub(crate) fn load_environment_variables_from_dot_env_file() {
    let env_map = TheoEnvMap::user_default();
    let env_file = PathBuf::project_path_buf().join(".env");
    if !env_file.exists() {
        std::fs::write(env_file.clone(), "").expect("Failed to write .env file.");
    }
    dotenv().expect("Failed to load .env file.");

    let mut missing_env_vars = false;
    for (env_key, env_value) in env_map.0.iter() {
        match env::var(env_key) {
            Ok(_) => {}
            Err(error) => {
                match error {
                    VarError::NotPresent => {
                        missing_env_vars = true;
                        println!("Environment variable '{}' is not set. Using a default value '{}'.", env_key, env_value);
                        let mut file = OpenOptions::new()
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
