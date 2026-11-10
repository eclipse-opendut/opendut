use leptos::prelude::*;

#[component]
pub fn InfoMessage(#[prop(into)] text: String) -> impl IntoView {

    view! {
        <article class="message is-info">
            <div class="message-body">
                <div class="columns is-mobile is-vcentered">
                    <div class="column is-narrow">
                        <i class="fa-solid fa-circle-info fa-lg"></i>
                    </div>
                    <div class="column">
                        <p>{ text }</p>
                    </div>
                </div>
            </div>
        </article>
    }
}
