use std::ops::Not;

use leptos::*;

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub text: String,
    pub href: String,
}

impl Breadcrumb {
    pub fn new(text: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            href: href.into(),
        }
    }
}

#[component]
pub fn Breadcrumbs(
    #[prop(into)] breadcrumbs: MaybeSignal<Vec<Breadcrumb>>,
) -> impl IntoView {

    let (items, back_items, _) = breadcrumbs.with(|breadcrumbs| breadcrumbs.iter()
        .enumerate()
        .fold((Vec::new(), Vec::new(), String::new()), |(mut result, mut href_result, mut base), (index, breadcrumb)| {

            base.push_str(&breadcrumb.href);

            let is_last = index == breadcrumbs.len() - 1;
            let text = Clone::clone(&breadcrumb.text);
            let href = Clone::clone(&base);
            let is_active = is_last;

            href_result.push(href.clone());
            result.push(view! { <Item text href is_active /> });

            if base.ends_with("/").not() && is_last.not() {
                base.push_str("/");
            }

            (result, href_result, base)
        })
    );

    view! {
        <nav class="is-hidden-tablet">
            <a href={ back_items.get(back_items.len()-2) } aria-label="breadcrumbs">Back</a>
        </nav>
        <nav class="breadcrumb mb-0 is-hidden-mobile" aria-label="breadcrumbs">
            <ul>
                { items }
            </ul>
        </nav>
    }
}

#[component]
fn Item(text: String, href: String, is_active: bool) -> impl IntoView {

    view! {
        <li class=("is-active", move || is_active)>
            <a href={ href }>{ text }</a>
        </li>
    }
}
