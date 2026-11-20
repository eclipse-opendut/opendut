use leptos::prelude::*;
use crate::components::{BasePageContainer, Breadcrumb};
use crate::downloads::{CleoCard, EdgarCard};

#[component(transparent)]
pub fn Downloads() -> impl IntoView {

    let breadcrumbs = {
        Signal::derive(move || {
            vec![
                Breadcrumb::new("Dashboard", "/"),
                Breadcrumb::new("Downloads", "/downloads"),
            ]
        })
    };

    view! {
        <BasePageContainer
            title="Downloads"
            breadcrumbs
            controls=view! { <> }
        >
            <div class="field">
                <div class="columns is-centered">
                    <div class="column">
                        <CleoCard />
                    </div>
                    <div class="column">
                        <EdgarCard />
                    </div>
                </div>
            </div>
        </BasePageContainer>
    }
}
