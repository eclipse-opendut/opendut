use serde::Deserialize;
use std::fs::File;
use std::io::Read;


#[derive(Deserialize, Debug)]
pub struct DockerConfig {
    // TODO: Add all docker configs from the main Config file
    port: u16,
    mode: bool,
    pub image_name: String
}

impl DockerConfig{
    pub fn get_image_name(&self) -> &str {
        &self.image_name
    }
}


#[derive(Deserialize, Debug)]
pub struct TestExecutionConfig {
    // TODO: Map config file to the structure
    request_id: String,
    response_id: String,
    interface: String,
    docker_config: DockerConfig,
    result_file_path: String,
}

impl TestExecutionConfig {
    pub fn from_file(file_path: &str) -> TestExecutionConfig {
        let mut file = File::open(file_path).expect("Failed to open the path");
        let mut contents = String::new();
        file.read_to_string(&mut contents);

        let config: TestExecutionConfig = serde_yaml::from_str(&contents).expect("Couldn't parse a yaml file");

        return config;
    }

    pub fn get_docker_config(&self) -> &DockerConfig {
        &self.docker_config
    }
}
