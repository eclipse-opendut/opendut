use leptos::{component, IntoView, MaybeSignal, view, create_local_resource};
use crate::app::{use_app_globals, ExpectGlobals};
use crate::components::{BasePageContainer, Breadcrumb, Initialized};
use crate::downloads::{CleoCard, EdgarCard};

#[component(transparent)]
pub fn Downloads() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {
        let globals = use_app_globals();
        
        let breadcrumbs = {
            MaybeSignal::derive(move || {
                vec![
                    Breadcrumb::new("Dashboard", "/"),
                    Breadcrumb::new("Downloads", "/downloads"),
                ]
            })
        };

        let version_info = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.metadata.version().await
                    .expect("Failed to request the version from carl.")
            }
        });

        view! {
            <BasePageContainer
                title="Downloads"
                breadcrumbs=breadcrumbs
                controls=view! { }
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

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}
