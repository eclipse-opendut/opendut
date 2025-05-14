use clap::Parser;
use tracing_subscriber::fmt::format::FmtSpan;

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
    pub fn run(self) -> anyhow::Result<()> {
        match self.device {
            DeviceKind::RaspberryPi(cli) => cli.run(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("opendut=trace")
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .with_writer(std::io::stderr) //allows piping stdout from task output without tracing interfering
        .init();

    let cli = FlashCli::parse();
    cli.run()
}
