pub const MOUSE_DRAG_PIXEL_THRESHOLD: f64 = 5.0;

pub mod view {
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
}

pub fn calculate_distance(start: (i32, i32), end: (i32, i32)) -> f64 {
    let (start_x, start_y) = start;
    let (end_x, end_y) = end;
    let diff_x = (end_x - start_x) as f64;
    let diff_y = (end_y - start_y) as f64;
    // euclidean distance formula -> distance = sqrt(diff_x² + diff_y²)
    (diff_x * diff_x + diff_y * diff_y).sqrt()
}
