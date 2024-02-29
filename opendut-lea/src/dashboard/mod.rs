use leptos::*;

use crate::clusters::ClustersCard;
use crate::components::Initialized;
use crate::peers::PeersCard;

#[component]
pub fn Dashboard() -> impl IntoView {

    view! {
        <Initialized>
            <div class="mt-6">
                <div class="columns is-full">
                    <div class="column">
                        <h1 class="title is-3 has-text-centered">Welcome</h1>
                    </div>
                </div>
                <div class="mt-5">
                    <div class="columns is-centered">
                        <div class="column is-one-third">
                            <ClustersCard />
                        </div>
                        <div class="column is-one-third">
                            <PeersCard />
                        </div>
                    </div>
                </div>
            </div>
        </Initialized>
    }
}
