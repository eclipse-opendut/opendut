use crate::service::execution::managers::Manager;
use crate::service::execution::managers::DockerManager;
// use crate::service::execution::managers::PythonEnvManager;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::execution::config_utils::TestExecutionConfig;
// use crate::service::execution::config_utils::Env;


pub enum ManagerType {
    Docker(DockerManager),
    // PythonEnv(PythonEnvManager)
}

impl ManagerType {
    pub fn create_manager(&self, test_config: &TestExecutionConfig) {
        match self {
            ManagerType::Docker(docker_manager) => {
                DockerManager::new(*test_config);
            }
            // ManagerType::PythonEnv(python_env_manager) => {
            //     PythonEnvManager::new(test_config);
            // }
        }
    }

    pub fn run_test(&self) {
        match self {
            ManagerType::Docker(docker_manager) => {
                docker_manager.run();
            }
            // ManagerType::PythonEnv(python_env_manager) => {
            //     python_env_manager.run();
            // }
        }
    }
}

pub struct TestExecutor {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub test_config: TestExecutionConfig,
    pub manager: ManagerType
}

impl TestExecutor{
    pub fn create(network_interface_manager: NetworkInterfaceManagerRef, config_file_path:String) {
        let test_config = TestExecutionConfig::from_file(&config_file_path);

        let manager =  ManagerType::Docker(DockerManager {config: test_config});
        // } else if test_config.get_environment().eq(Env::Python){
        //     // Create Python env manager
        //     log::trace!("Python env was selected");
        // } else {
        //     log::trace!("Not a valid environment was selected");
        // }

        manager.create_manager(&test_config);

        Self {network_interface_manager, test_config, manager};
    }   

    pub fn run(&self) {
        self.manager.run_test()
    } 
}
