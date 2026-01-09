pub fn render_error_message<T: std::error::Error>(fail: &T, msg: &'static str) -> String {
    let mut err_msg = format!("{}\nError sources: ", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    Caused by: {}", cause);
        cur_fail = cause.source();
    }
    err_msg
}
