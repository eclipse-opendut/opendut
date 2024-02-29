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

    let breadcrumb_items= MaybeSignal::derive(move || {
        breadcrumbs.with(|breadcrumbs| breadcrumbs.iter()
            .enumerate()
            .fold((Vec::new(), String::new()), |(mut result, mut base), (index, breadcrumb)| {

                base.push_str(&breadcrumb.href);

                let is_last = index == breadcrumbs.len() - 1;
                let text = Clone::clone(&breadcrumb.text);
                let href = Clone::clone(&base);
                let is_active = is_last;

                result.push(view! { <Item text href is_active /> });

                if base.ends_with('/').not() && is_last.not() {
                    base.push('/');
                }

                (result, base)
            })
        )
    });

    view! {
         <nav class="breadcrumb mb-0 is-hidden-tablet" aria-label="backButton">
            <ul>
                { breadcrumb_items.get().0.iter().nth_back(1) }
                <span class="icon ml-0">
                    <i class="fa-solid fa-arrow-left"></i>
                </span>
            </ul>
        </nav>
        <nav class="breadcrumb mb-0 is-hidden-mobile" aria-label="breadcrumbs">
            <ul>
                { breadcrumb_items.get().0 }
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
