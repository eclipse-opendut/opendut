use leptos::prelude::*;
use crate::components::BasePageContainer;

use shadow_rs::shadow;
use crate::util::NON_BREAKING_SPACE;
shadow!(build);

#[component]
pub fn AboutOverview() -> impl IntoView {
    view! {
        <BasePageContainer
            title="About"
            breadcrumbs=Vec::new()
            controls=view! { <> }
        >
            <div class="mt-4">
                <table class="table">
                    <tbody>
                        <tr>
                            <td>Version</td>
                            <td>{ build::PKG_VERSION }</td>
                        </tr>
                        <tr>
                            <td>Revision</td>
                            <td>{ build::COMMIT_HASH }</td>
                        </tr>
                        <tr>
                            <td>Revision Date</td>
                            <td>{ build::COMMIT_DATE }</td>
                        </tr>
                        <tr>
                            <td>Build Date</td>
                            <td>{ build::BUILD_TIME }</td>
                        </tr>
                    </tbody>
                </table>

                <a href="https://opendut.eclipse.dev/"><i class="fa-solid fa-arrow-up-right-from-square"></i>{ NON_BREAKING_SPACE } openDuT Project Overview</a>
            </div>
        </BasePageContainer>
    }
}
