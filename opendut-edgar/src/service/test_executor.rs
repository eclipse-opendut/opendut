use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub struct TestExecutor {
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

impl TestExecutor{
    pub fn create(network_interface_manager: NetworkInterfaceManagerRef) -> Self {
        Self {network_interface_manager}
    }

    
}