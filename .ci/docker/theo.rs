#!/usr/bin/env rust-script
//! Dependencies can be specified in the script file itself as follows:
//!
//! ```cargo
//! [dependencies]
//! clap = { version = "4.4.8", features = ["derive", "wrap_help", "env"] }
//! dotenv = "0.15.0"
//! strum = { version = "0.25.0", features = ["derive"] }
//! strum_macros = { version = "0.25.3" }
//! serde = { version = "1.0.193", features = ["derive"] }
//! serde_json = "1.0.108"
//! phf = { version = "0.11", features = ["macros"] }
//! ```


extern crate clap;
extern crate dotenv;

use phf::phf_map;
use std::net::Ipv4Addr;
use serde::{de, Deserialize, Deserializer};
use std::env;
use std::fs;
use std::process::{Command, Output};
use std::collections::HashMap;
use std::ops::Index;
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use strum::VariantNames;
use strum_macros::{EnumString, EnumVariantNames, Display};

fn project_root_dir() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .expect("Failed to determine git project root directory");
    consume_output(output)
}

fn check_docker_compose_is_installed() {
    let output = Command::new("docker")
        .arg("compose")
        .arg("version")
        .status()
        .expect("Failed to run docker compose. Check if docker compose plugin is installed. \
                See https://docs.docker.com/compose/install/linux/ for instructions.");
    assert!(output.success());
}

enum DockerCoreServices {
    Network,
    Carl,
    Keycloak,
    Edgar,
    Netbird,
    Firefox,
}

impl DockerCoreServices {
    fn as_str(&self) -> &'static str {
        match self {
            DockerCoreServices::Carl => "carl",
            DockerCoreServices::Keycloak => "keycloak",
            DockerCoreServices::Edgar => "edgar",
            DockerCoreServices::Netbird => "netbird",
            DockerCoreServices::Network => "network",
            DockerCoreServices::Firefox => "firefox",
        }
    }
}

enum DockerDeveloperServices {
    Firefox,
    Dev,
}

impl DockerDeveloperServices {
    fn as_str(&self) -> &'static str {
        match self {
            DockerDeveloperServices::Firefox => "firefox",
            DockerDeveloperServices::Dev => "dev",
        }
    }
}

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

/// Run docker compose up from project root for compose-config-file in given directory.
///
/// Example:
///
///   docker compose -f ./.ci/docker/<compose_dir>/docker-compose.yml --env-file .env up -d
fn add_common_docker_args(command: &mut Command, compose_dir: &str) {
    command
        .arg("compose")
        .arg("-f")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", compose_dir))
        .arg("--env-file")
        .arg(".env");
}

fn docker_compose_up(compose_dir: &str) {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, compose_dir);
    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

fn docker_compose_build(compose_dir: &str) {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, compose_dir);
    let command_status = command
        .arg("build")
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute docker compose build for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

fn docker_compose_down(compose_dir: &str, delete_volumes: bool) {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, compose_dir);
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    let command_status = command
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

fn docker_compose_network_create() {
    let output = Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg("./.ci/docker/network/docker-compose.yml")
        .arg("up")
        .arg("--force-recreate")
        .current_dir(project_root_dir())
        .status()
        .expect("Failed to create docker network.");

    assert!(output.success());
}

fn docker_compose_network_delete() {
    let output = Command::new("docker")
        .arg("network")
        .arg("rm")
        .arg("opendut_network")
        .status()
        .expect("Failed to create docker network.");

    assert!(output.success());
}

fn ip_address_from_str<'de, D>(deserializer: D) -> Result<Ipv4Addr, D::Error>
    where D: Deserializer<'de>
{
    let string = String::deserialize(deserializer)?;
    let ip_string = string
        .trim_matches('\"')
        .trim_end_matches("/24")
        .to_string();
    ip_string.parse::<Ipv4Addr>().map_err(de::Error::custom)
    //ipaddress::IPAddress::parse(&s).map_err(de::Error::custom)
}

#[derive(Debug, Deserialize)]
struct ContainerAddress {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "EndpointID")]
    endpoint_id: String,
    #[serde(rename = "MacAddress")]
    mac_address: String,
    #[serde(rename = "IPv4Address", deserialize_with = "ip_address_from_str")]
    ipv4address: Ipv4Addr,
    #[serde(rename = "IPv6Address")]
    ipv6address: String,
}

enum DockerHostnames {
    Carl,
    Keycloak,
    Edgar,
    NetbirdManagement,
    NetbirdDashboard,
    NetbirdSignal,
    NetbirdCoturn,
    Firefox,
}

impl DockerHostnames {
    fn as_str(&self) -> &'static str {
        match self {
            DockerHostnames::Carl => "carl",
            DockerHostnames::Keycloak => "keycloak",
            DockerHostnames::Edgar => "edgar",
            DockerHostnames::NetbirdManagement => "netbird-management",
            DockerHostnames::NetbirdDashboard => "netbird-dashboard",
            DockerHostnames::NetbirdSignal => "netbird-signal",
            DockerHostnames::NetbirdCoturn => "netbird-coturn",
            DockerHostnames::Firefox => "firefox",
        }
    }
}
static CONTAINER_NAME_MAP: phf::Map<&'static str, DockerHostnames> = phf_map! {
    "firefox" => DockerHostnames::Firefox,
    "keycloak-keycloak-1" => DockerHostnames::Keycloak,
    "carl-carl-1" => DockerHostnames::Carl,
    "netbird-management-1" => DockerHostnames::NetbirdManagement,
    "netbird-dashboard-1" => DockerHostnames::NetbirdDashboard,
};

fn docker_inspect_network() {
    println!("# BEGIN OpenDuT docker network 'docker network inspect opendut_network'");
    let output = Command::new("docker")
        .arg("network")
        .arg("inspect")
        .arg("opendut_network")
        .arg("--format")
        .arg("'{{json .Containers}}'")
        .output()
        .expect("Failed to inspect docker network.");

    let stdout = consume_output(output).trim_matches('\'').to_string();
    let opendut_container_address_map: HashMap<String, ContainerAddress> =
        serde_json::from_str(&stdout).expect("JSON was not well-formatted");
    let mut sorted_addresses: Vec<(&String, &ContainerAddress)> = opendut_container_address_map.iter().collect();
    sorted_addresses
        .sort_by(|a, b| a.1.ipv4address.cmp(&b.1.ipv4address));

    for (key, value) in &sorted_addresses {
        let ip_address = value.ipv4address.to_string();
        let given_hostname = value.name.clone();
        let hostname = if CONTAINER_NAME_MAP.contains_key(given_hostname.as_str()) {
            CONTAINER_NAME_MAP.get(given_hostname.as_str()).unwrap().as_str().to_string()
        } else {
            given_hostname
        };

        let padding = std::cmp::max(0, 20 - ip_address.clone().len());
        let whitespace = std::iter::repeat(" ").take(padding).collect::<String>();
        let padded_ip_address = ip_address.clone() + &whitespace;
        println!("{}  {}", padded_ip_address, hostname);
    }
    println!("# END OpenDuT docker network 'opendut_network'");
}

fn consume_output(output: Output) -> String {
    output.stdout
        .iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim()
        .to_string()
}

fn build_testenv() {
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_network_create();
    docker_compose_build(DockerCoreServices::Firefox.as_str());
    docker_compose_build(DockerCoreServices::Keycloak.as_str());
    docker_compose_build(DockerCoreServices::Carl.as_str());
    docker_compose_build(DockerCoreServices::Netbird.as_str());

}

fn start_testenv() {
    // prerequisites
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_network_create();

    // start services
    docker_compose_up(DockerCoreServices::Firefox.as_str());
    docker_compose_up(DockerCoreServices::Keycloak.as_str());
    docker_compose_up(DockerCoreServices::Carl.as_str());
    docker_compose_up(DockerCoreServices::Netbird.as_str());

    // TODO: start edgar requires additional steps to run in managed mode
    println!("Go to OpenDuT Browser at http://localhost:3000/")
}

fn stop_testenv() {
    // prerequisites
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_down(DockerCoreServices::Keycloak.as_str(), false);
    docker_compose_down(DockerCoreServices::Carl.as_str(), false);
    docker_compose_down(DockerCoreServices::Netbird.as_str(), false);
    docker_compose_down(DockerCoreServices::Firefox.as_str(), false);
}

fn destroy_testenv() {
    // prerequisites
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_down(DockerCoreServices::Keycloak.as_str(), true);
    docker_compose_down(DockerCoreServices::Carl.as_str(), true);
    docker_compose_down(DockerCoreServices::Netbird.as_str(), true);
    docker_compose_down(DockerCoreServices::Firefox.as_str(), true);
    docker_compose_network_delete();
}


#[derive(Debug, Parser)]
#[command(name = "opendut-theo")]
#[command(about = "opendut-theo - Test harness environment operator.")]
#[command(long_version = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Build docker containers.")]
    Build,
    #[command(about = "Start test environment.")]
    Start,
    #[command(about = "Stop test environment.")]
    Stop,
    #[command(about = "Show docker network.")]
    Network,
    #[command(about = "Destroy test environment.")]
    Destroy,
}

fn get_docker_group_id() -> String {
    let docker_getent_group = consume_output(Command::new("getent").arg("group").arg("docker").output().expect("Failed to get group id"));
    let docker_group_id = docker_getent_group.split(":").nth(2).expect("Failed to get docker group id").to_string();
    docker_group_id
}

fn check_dot_env_variables() {
    let existing_env_keys = env::vars().map(|(key, _)| key).collect::<Vec<String>>();
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

fn enumerate_distribution_tar_files(dist_path: PathBuf) -> Vec<String> {
    let paths = fs::read_dir(dist_path).unwrap();
    let files = paths.into_iter()
        .map(|path| { path.unwrap().path() })
        .filter(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            file_name.contains("opendut") && file_name.contains(".tar.gz")
        })
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    files
}

fn assert_exactly_one_distribution_of_each_component(expected_dist_files: &Vec<&str>, files: &Vec<String>) {
    for expected in expected_dist_files.clone() {
        let filtered_existing_files = files.iter().cloned()
            .filter(|file| file.contains(expected))
            .collect::<Vec<_>>();
        assert_eq!(filtered_existing_files.len(), 1,
                   "There should be exactly one dist of '{}'. Found: {:?}", expected, filtered_existing_files);
    }
}

const TARGET_TRIPLE: &'static str = "x86_64-unknown-linux-gnu";

fn check_if_distribution_tar_exists_of_each_component(expected_dist_files: &Vec<&str>, files: Vec<String>) -> bool {
    let stripped_version_of_files = files.iter().cloned()
        .map(|file| {
            let pos = file.find(TARGET_TRIPLE).map(|i| i + 12).unwrap();
            file.index(..pos).to_owned()
        })
        .collect::<Vec<_>>();

    let count_existing_dist_files = expected_dist_files.iter().cloned().map(|expected| {
        stripped_version_of_files.contains(&expected.to_owned())
    });
    count_existing_dist_files.len() != expected_dist_files.len()
}

fn make_distribution_with_cargo() {
    println!("Create distribution with cargo: 'cargo ci distribution'");
    let dist_status = Command::new("cargo")
        .arg("ci")
        .arg("distribution")
        .status()
        .expect("Failed to update distribution");
}

fn make_distribution_if_not_present() {
    let root_dir = project_root_dir();
    let dist_directory_path = Path::new(root_dir.as_str())
        .join(format!("target/ci/distribution/{}", TARGET_TRIPLE));
    let expected_dist_files = vec!(
        //"opendut-cleo-linux-x86_64",
        "opendut-edgar-x86_64-unknown-linux-gnu",
        "opendut-carl-x86_64-unknown-linux-gnu",
    );

    if !dist_directory_path.exists() {
        make_distribution_with_cargo();
    }

    let present_dist_files = enumerate_distribution_tar_files(dist_directory_path);
    assert_exactly_one_distribution_of_each_component(&expected_dist_files, &present_dist_files);

    if check_if_distribution_tar_exists_of_each_component(&expected_dist_files, present_dist_files) {
        make_distribution_with_cargo();
    }

}

fn main() {
    dotenv().ok();
    check_dot_env_variables();
    let args = Cli::parse();

    match args.command {
        Commands::Build => {
            println!("Building testenv");
            make_distribution_if_not_present();
            build_testenv();
        }
        Commands::Start => {
            make_distribution_if_not_present();

            println!("Starting testenv");
            start_testenv();
        }
        Commands::Stop => {
            println!("Stopping testenv");
            stop_testenv();
        }
        Commands::Network => {
            docker_inspect_network();
        }
        Commands::Destroy => {
            println!("Destroying testenv");
            destroy_testenv();
        }
    }
}
