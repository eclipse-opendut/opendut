use leptos::prelude::*;

pub fn join_with_comma_spans<T: RenderHtml + 'static>(elements: Vec<View<T>>) -> Vec<AnyView> {
    let elements_length = elements.len();

    let mut elements_with_separator = Vec::new();

    for (index, element) in elements.into_iter().enumerate() {
        elements_with_separator.push(element.into_any());

        if index < (elements_length - 1) {
            elements_with_separator.push(
                view! { <span>", "</span> }.into_any()
            );
        }
    }
    elements_with_separator
}
