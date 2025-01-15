use leptos::prelude::*;
use crate::app::use_app_globals;
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
        
        let metadata: LocalResource<VersionInfo> = LocalResource::new(move || {
            let carl = globals.client.clone();
            async move {
                let mut carl = carl.clone();
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
                                <td>LEA</td>
                                <td>Version</td>
                                <td>{ build::PKG_VERSION }</td>
                            </tr>
                            <tr>
                                <td rowspan="4">CARL</td>
                                <td>Version</td>
                                <td>
                                    <Transition fallback=move || view! { <p>"-"</p> }>
                                        { move || Suspend::new(async move { metadata.await.name })}
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
                    <a href="https://opendut.eclipse.dev/"><i class="fa-solid fa-arrow-up-right-from-square"></i>{ NON_BREAKING_SPACE } openDut Project Overview</a>
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
