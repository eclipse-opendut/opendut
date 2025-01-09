use leptos::prelude::*;

pub fn join_with_comma_spans(elements: Vec<View>) -> Vec<View> {
    let elements_length = elements.len();

    let mut elements_with_separator = Vec::new();

    for (index, element) in elements.into_iter().enumerate() {
        elements_with_separator.push(element);

        if index < (elements_length - 1) {
            elements_with_separator.push(
                view! { <span>", "</span> }
                    .into_view()
            );
        }
    }
    elements_with_separator
}
