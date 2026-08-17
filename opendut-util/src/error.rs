pub fn render_error_message<T: std::error::Error>(fail: &T, msg: &'static str) -> String {
    let mut err_msg = format!("{}\nError sources: ", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(source) = cur_fail {
        err_msg += &format!("\n    Caused by: {}", source);
        cur_fail = source.source();
    }
    err_msg
}
