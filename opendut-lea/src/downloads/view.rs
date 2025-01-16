use leptos::prelude::*;
use crate::app::use_app_globals;
use crate::components::{BasePageContainer, Breadcrumb, Initialized};
use crate::downloads::{CleoCard, EdgarCard};

#[component(transparent)]
pub fn Downloads() -> impl IntoView {

    let globals = use_app_globals();

    let breadcrumbs = {
        Signal::derive(move || {
            vec![
                Breadcrumb::new("Dashboard", "/"),
                Breadcrumb::new("Downloads", "/downloads"),
            ]
        })
    };

    let version_info = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        async move {
            carl.metadata.version().await
                .expect("Failed to request the version from carl.")
        }
    });

    view! {
        <BasePageContainer
            title="Downloads"
            breadcrumbs=breadcrumbs
            controls=view! { <> }
        >
            <div class="field">
                <div class="columns is-centered">
                    <div class="column">
                        <CleoCard version_info/>
                    </div>
                    <div class="column">
                        <EdgarCard version_info/>
                    </div>
                </div>
            </div>
        </BasePageContainer>
    }
}
