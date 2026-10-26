use clap::ValueEnum;

#[derive(ValueEnum, Clone)]
pub(crate) enum OutputFormat {
    Table,
    Json,
    PrettyJson,
}
