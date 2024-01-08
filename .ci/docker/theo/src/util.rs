use std::process::Output;

pub(crate) fn consume_output(output: Output) -> String {
    output.stdout
        .iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim()
        .to_string()
}
