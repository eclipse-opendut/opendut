use leptos::web_sys;

pub fn has_text_selection() -> bool {
    let window = web_sys::window().unwrap();
    if let Ok(Some(sel)) = window.get_selection() {
        !sel.is_collapsed()
    } else {
        false
    }
}
