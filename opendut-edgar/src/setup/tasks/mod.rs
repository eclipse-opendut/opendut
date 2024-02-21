mod check_carl_reachable;
pub use check_carl_reachable::CheckCarlReachable;

mod check_os_requirements;
pub use check_os_requirements::CheckOsRequirements;

mod claim_file_ownership;
pub use claim_file_ownership::ClaimFileOwnership;

mod copy_executable;
pub use copy_executable::CopyExecutable;

mod create_user;
pub use create_user::CreateUser;

mod create_service;
pub use create_service::CreateServiceFile;

/// EDGAR Service needs to modify network interfaces.
/// This module contains tasks to request the Linux Capability "CAP_NET_ADMIN", which allows doing so without root permissions.
pub mod linux_network_capability;

pub mod netbird;

pub mod network_interface;

mod start_service;
pub use start_service::StartService;

pub mod write_configuration;
pub use write_configuration::WriteConfiguration;

pub mod load_kernel_modules;
pub use load_kernel_modules::LoadKernelModules;

pub mod create_kernel_module_load_rule;
pub use create_kernel_module_load_rule::CreateKernelModuleLoadRule;
