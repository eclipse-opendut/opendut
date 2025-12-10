use leptos::prelude::*;
use crate::peers::configurator::types::executor::{UserPeerExecutor, UserPeerExecutorKind};
use crate::peers::configurator::types::UserPeerConfiguration;

impl UserPeerConfiguration {
    pub fn is_valid(&self) -> bool {
        self.valid_general_tab()
            && self.valid_devices_tab()
                && self.valid_executor_tab()
    }

    pub fn valid_general_tab(&self) -> bool {
        self.name.is_right()
            && self.location.is_right()
    }

    pub fn valid_devices_tab(&self) -> bool {
        self.devices.iter().all(|device_configuration| {
            let device_configuration = device_configuration.get();
            device_configuration.name.is_right()
                && device_configuration.interface.is_some()
        })
    }

    pub fn valid_executor_tab(&self) -> bool {
        self.executors.iter().all(|executor| {
            let executor = executor.get();
            let UserPeerExecutor { id: _, kind, results_url, is_collapsed: _ } = executor;

            let kind_is_valid = match kind {
                UserPeerExecutorKind::Container {
                    engine: _,
                    name,
                    image,
                    volumes,
                    devices,
                    envs,
                    ports,
                    command,
                    args,
                } => {
                    name.is_right()
                        && image.is_right()
                        && volumes.iter().all(|volume| volume.get().is_right())
                        && devices.iter().all(|device| device.get().is_right())
                        && envs.iter().all(|env| env.get().name.is_right())
                        && ports.iter().all(|port| port.get().is_right())
                        && command.is_right()
                        && args.iter().all(|arg| arg.get().is_right())
                }
            };

            kind_is_valid && results_url.is_right()
        })
    }
}
