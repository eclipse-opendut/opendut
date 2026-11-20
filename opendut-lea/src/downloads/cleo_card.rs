use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::components::{GenerateSetupStringForm, GenerateSetupStringKind};

#[component]
pub fn CleoCard() -> impl IntoView {

    view! {
        <div class="card">
            <div class="card-header">
                <div class="card-header-title"><i class="fa-solid fa-terminal mr-1"></i>"CLEO"</div>
            </div>
            <div class="card-content">
                <InfoMessage text="Command-line interface for administering openDuT." />
                "Download for different architectures:"
                <div class="mb-2 mt-2 ml-2">
                    <a href="/api/cleo/aarch64-unknown-linux-gnu/download" download>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"aarch64-gnu"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/cleo/armv7-unknown-linux-gnueabihf/download" download>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"armv7-gnueabihf"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/cleo/x86_64-unknown-linux-gnu/download" download>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"x86_64-gnu"</span>
                    </a>
                </div>
                <div class="mt-5">
                    <div class="field">
                        <GenerateSetupStringForm kind={Signal::derive(move || GenerateSetupStringKind::Cleo)} />
                    </div>
                </div>
            </div>
        </div>
    }
}
