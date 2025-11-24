use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb};
use crate::sources::components::CreateSourceButton;

#[component(transparent)]
pub fn SourcesOverview() -> impl IntoView {

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Sources", "/sources")
    ];

    view! {
        <BasePageContainer
            title="Sources"
            breadcrumbs=breadcrumbs
            controls=view! {
                <CreateSourceButton />
            }
        >

        <div>
            Hallo
        </div>

        </BasePageContainer>
    }
}
