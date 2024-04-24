use crate::{DecodePeerSetupOutputFormat, ParseablePeerSetup};

/// Decode the setup string of a peer
#[derive(clap::Parser)]
pub struct DecodePeerSetupCli {
    ///Setup string
    #[arg()]
    setup_string: ParseablePeerSetup,
    ///Text, JSON or prettified JSON as output format
    #[arg(value_enum, short, long, default_value_t=DecodePeerSetupOutputFormat::Json)]
    output: DecodePeerSetupOutputFormat,
}

impl DecodePeerSetupCli {
    pub async fn execute(self) -> crate::Result<()> {
        let setup_string = *self.setup_string.0;
        let text = match self.output {
            DecodePeerSetupOutputFormat::Text => {
                format!("{:#?}", setup_string)
            }
            DecodePeerSetupOutputFormat::Json => serde_json::to_string(&setup_string).unwrap(),
            DecodePeerSetupOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&setup_string).unwrap()
            }
        };
        println!("{text}");
        Ok(())
    }
}