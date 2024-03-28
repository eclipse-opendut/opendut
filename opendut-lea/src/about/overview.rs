use leptos::*;
use crate::app::{ExpectGlobals, use_app_globals};
use crate::components::{BasePageContainer, Initialized};

use shadow_rs::shadow;
use opendut_types::proto::util::VersionInfo;
use crate::util::NON_BREAKING_SPACE;
shadow!(build);

#[component]
pub fn AboutOverview() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {
        
        let globals = use_app_globals();
        
        let metadata: Resource<(), VersionInfo> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.metadata.version().await
                    .expect("Failed to request the version from carl.")
            }
        });
        
        view! {
            <BasePageContainer
                title="About"
                breadcrumbs=Vec::new()
                controls=view! { }
            >
                <div class="mt-4">
                    <table class="table is-bordered">
                        <tbody>
                             <tr>
                                <td>Lea</td>
                                <td>Version</td>
                                <td>{ build::PKG_VERSION }</td>
                            </tr>
                            <tr>
                                <td rowspan="4">Carl</td>
                                <td>Version</td>
                                <td>
                                    <Transition fallback=move || view! { <p>"-"</p> }>
                                        { move || { metadata.get().map(|version| version.name)} }
                                    </Transition>
                                </td>
                            </tr>
                            <tr>
                                <td>Revision</td>
                                <td>
                                    <Transition fallback=move || view! { <p>"-"</p> }>
                                        { move || { metadata.get().map(|version| version.revision)} }
                                    </Transition>
                                </td>
                            </tr>
                            <tr>
                                <td>Revision Date</td>
                                <td>
                                    <Transition fallback=move || view! { <p>"-"</p> }>
                                        { move || { metadata.get().map(|version| version.revision_date)} }
                                    </Transition>
                                </td>
                            </tr>
                            <tr>
                                <td>Build Date</td>
                                <td>
                                    <Transition fallback=move || view! { <p>"-"</p> }>
                                        { move || { metadata.get().map(|version| version.build_date)} }
                                    </Transition>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                    <a href="https://opendut.eclipse.dev/"><i class="fa-solid fa-arrow-up-right-from-square"></i>{ NON_BREAKING_SPACE } OpenDut Project Overview</a>
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