use crate::{DecodeSetupStringOutputFormat, ParseableSetupString};

/// Decode the setup string of a peer
#[derive(clap::Parser)]
pub struct DecodeSetupStringCli {
    ///Setup string
    #[arg()]
    setup_string: ParseableSetupString,
    ///Text, JSON or prettified JSON as output format
    #[arg(value_enum, short, long, default_value_t=DecodeSetupStringOutputFormat::Json)]
    output: DecodeSetupStringOutputFormat,
}

impl DecodeSetupStringCli {
    pub async fn execute(self) -> crate::Result<()> {
        let setup_string = *self.setup_string.0;
        let text = match self.output {
            DecodeSetupStringOutputFormat::Text => {
                format!("{:#?}", setup_string)
            }
            DecodeSetupStringOutputFormat::Json => serde_json::to_string(&setup_string).unwrap(),
            DecodeSetupStringOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&setup_string).unwrap()
            }
        };
        println!("{text}");
        Ok(())
    }
}
