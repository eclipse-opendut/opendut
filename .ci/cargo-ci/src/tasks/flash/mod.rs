mod raspberry_pi;

///Flash a device with an operating system image
#[derive(clap::Parser)]
pub struct FlashCli {
    /// The kind of device to flash
    #[command(subcommand)]
    device: DeviceKind,
}

#[derive(clap::Subcommand)]
enum DeviceKind {
    RaspberryPi(raspberry_pi::RaspberryPiCli),
}

impl FlashCli {
    #[tracing::instrument(name="flash", skip(self))]
    pub fn default_handling(self) -> crate::Result {
        match self.device {
            DeviceKind::RaspberryPi(cli) => cli.run(),
        }
    }
}
