use leptos::*;
use crate::app::use_app_globals;

use crate::clusters::components::CreateClusterButton;

#[component]
pub fn ClustersCard() -> impl IntoView {

    let _globals = use_app_globals();

    view! {
        <div class="card">
            <div class="card-header">
                <a class="card-header-title has-text-link" href="/clusters">"Clusters"</a>
            </div>
            <div class="card-content">
                <div class="level">
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">Deployed</p>
                            <p class="title">
                                -
                            </p>
                        </div>
                    </div>
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">Undeployed</p>
                            <p class="title">
                                -
                            </p>
                        </div>
                    </div>
                </div>
            </div>
            <div class="card-footer">
                <div class="m-2">
                    <CreateClusterButton />
                </div>
            </div>
        </div>
    }
}
