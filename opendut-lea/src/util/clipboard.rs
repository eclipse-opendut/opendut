use leptos::{create_action, window, Action};
use crate::components::{use_toaster, Toast};

pub fn copy_with_feedback() -> Action<String, ()> {
    create_action(move |clipboard_text: &String| {
        let toaster = use_toaster();
        let test = clipboard_text.clone();
        async move {
            let clipboard = window().navigator().clipboard();
            let clipboard_promise = clipboard.write_text(&test);
            match wasm_bindgen_futures::JsFuture::from(clipboard_promise).await {
                Ok(_) => {
                    toaster.toast(
                        Toast::builder()
                            .simple("Successfully copied Setup-String.")
                            .success(),
                    );
                }
                Err(_) => {
                    toaster.toast(
                        Toast::builder()
                            .simple("Error while copying Setup-String.")
                            .error(),
                    );
                }
            };
        }
    })
}
