use crate::DecodePeerSetupOutputFormat;
use opendut_types::peer::PeerSetup;

pub async fn execute(
    setup: PeerSetup,
    output: DecodePeerSetupOutputFormat,
) -> crate::Result<()> {
    let text = match output {
        DecodePeerSetupOutputFormat::Text => {
            format!("{:#?}", setup)
        }
        DecodePeerSetupOutputFormat::Json => serde_json::to_string(&setup).unwrap(),
        DecodePeerSetupOutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&setup).unwrap()
        }
    };
    println!("{text}");
    Ok(())
}
