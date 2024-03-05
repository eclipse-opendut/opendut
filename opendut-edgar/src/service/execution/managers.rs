use tokio::process::Command;
use crate::service::execution::config_utils::TestExecutionConfig;


pub(crate) trait Manager {
    fn new(env_config: TestExecutionConfig) -> Self;
    fn run(&self);
}

pub struct DockerManager {
    pub config: TestExecutionConfig
}

// pub struct PythonEnvManager {
//     pub config: TestExecutionConfig
// }

impl Manager for DockerManager{

    fn new(env_config: TestExecutionConfig) -> DockerManager {
        DockerManager {config: env_config}
    }

    fn run(&self) {

        self.load_docker_image_from_machine();
        
        // Run the Docker container from the built image, TOOD; figure out how to dynamically add arguments if not ran by default, for example, a special port is needed or binding
        let run_output = Command::new("docker")
            .args(&["run", "--rm", self.config.get_docker_config().get_image_name()])
            .output();    
        // Check if the container run was successful
    }
}

impl DockerManager {
    
    fn load_docker_image_from_machine(&self) {
        let run_output = Command::new("docker").args(&["load", "-i", self.config.get_docker_config().get_image_name()]).output();
    }

    fn pull_docker_image_from_registry(&self) {
        let run_output = Command::new("docker").args(&["pull", "-i", self.config.get_docker_config().get_image_name()]).output();
    }
    
    fn move_can_channel_to_docker_namespace(docker_pid: &str, can_interface: &str, bitrate:  &str) {
        // Execute: sudo ip link set can0 netns $DOCKERPID
        let output = Command::new("sudo").args(&["ip", "link", "set", can_interface, "netns", docker_pid])
        .output();

        // Execute : sudo nsenter -t $DOCKERPID -n ip link set can0 type can bitrate 500000
        let output = Command::new("sudo").args(&["nsenter", "-t", docker_pid, "-n", "ip", "link", "set", can_interface, "type", "can", "bitrate", bitrate])
        .output();

        // Execute: sudo nsenter -t $DOCKERPID -n ip link set can0 up
        let output = Command::new("sudo").args(&["nsenter", "-t", docker_pid, "-n", "ip", "link", "set", can_interface, "up"])
        .output();
    
    }

    fn get_files_from_docker() {
        println!("TODO");
    }

}


// impl Manager for PythonEnvManager{
//     fn new(env_config: TestExecutionConfig) -> PythonEnvManager {
//         PythonEnvManager {config: env_config}
//     }

//     fn run(&self) {}

// }

// impl PythonEnvManager{
//     fn setup_env(){}
// }


// fn main() {
//     let test_config = TestExecutionConfig::from_file("home/ubuntu/example_config.yaml");
//     let manager = DockerManager::new(test_config);
//     manager.run()
// }