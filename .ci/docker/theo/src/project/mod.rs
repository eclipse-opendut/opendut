pub(crate) mod make_dist;

use std::collections::HashMap;
use std::env;
use std::process::Command;
use crate::util::consume_output;
// instead of use strum_macros::{EnumString, EnumVariantNames, Display};
use strum::{VariantNames, EnumVariantNames, EnumString, Display};

pub(crate) const TARGET_TRIPLE: &'static str = "x86_64-unknown-linux-gnu";


#[derive(Debug, PartialEq, EnumString, EnumVariantNames, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvVars {
    Puser,
    Pgroup,
    Puid,
    Pgid,
    DockerUser,
    DockerGid,
    OpendutRepoRoot,
}

pub(crate) fn project_root_dir() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .expect("Failed to determine git project root directory");
    consume_output(output)
}

fn get_docker_group_id() -> String {
    let docker_getent_group = consume_output(Command::new("getent").arg("group").arg("docker").output().expect("Failed to get group id"));
    let docker_group_id = docker_getent_group.split(":").nth(2).expect("Failed to get docker group id").to_string();
    docker_group_id
}

pub(crate) fn check_dot_env_variables() {
    let user_name = consume_output(Command::new("id").arg("--user").arg("--name").output().expect("Failed to get user name"));
    let user_id = consume_output(Command::new("id").arg("--user").output().expect("Failed to get user id"));
    let group_name = consume_output(Command::new("id").arg("--group").arg("--name").output().expect("Failed to get group name"));
    let group_id = consume_output(Command::new("id").arg("--group").output().expect("Failed to get group id"));
    let docker_gid = get_docker_group_id();
    let git_repo_root = project_root_dir();

    let mut missing_env_vars = "".to_owned();
    let mut error_messages = "".to_owned();

    let env_map = HashMap::from([
        (EnvVars::Puser.to_string(), user_name.clone()),
        (EnvVars::Puid.to_string(), user_id.clone()),
        (EnvVars::Pgroup.to_string(), group_name.clone()),
        (EnvVars::Pgid.to_string(), group_id.clone()),
        (EnvVars::DockerUser.to_string(), format!("{}:{}", user_id, group_id)),
        (EnvVars::DockerGid.to_string(), docker_gid.clone()),
        (EnvVars::OpendutRepoRoot.to_string(), git_repo_root.clone()),
    ]);

    for (env_key, env_value) in &env_map {
        match env::var(env_key) {
            Ok(value) => {
                // check if all environment variables are set correctly
                if value != *env_value {
                    let wrong_key_value = format!("Env variable is set as '{}={}'. Expected: '{}={}'\n", env_key, value, env_key, env_value);
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

    assert_eq!(["PUSER", "PGROUP", "PUID", "PGID", "DOCKER_USER", "DOCKER_GID", "OPENDUT_REPO_ROOT"], EnvVars::VARIANTS);
}
