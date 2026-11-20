use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::routing;

#[component]
pub fn EdgarCard() -> impl IntoView {

    view! {
        <div class="card">
            <div class="card-header">
                <div class="card-header-title"><i class="fa-solid fa-microchip mr-1"></i>"EDGAR"</div>
            </div>
            <div class="card-content">
                <InfoMessage text="Edge software for integrating ECUs into openDuT." />
                "Download for different architectures:"
                <div class="mt-2 mb-2 ml-2">
                    <a href="/api/edgar/aarch64-unknown-linux-gnu/download" download="">
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"aarch64-gnu"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/edgar/armv7-unknown-linux-gnueabihf/download" download="">
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"armv7-gnueabihf"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/edgar/x86_64-unknown-linux-gnu/download" download="">
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"x86_64-gnu"</span>
                    </a>
                </div>
                <div class="mt-5">
                    <div class="field">
                        <label class="label">Setup-String</label>
                        <div class="mt-2">"Setup-Strings for each peer can be retrieved from the peer's specific configuration page."</div>
                        <div class="mt-2">
                            "To do so, select the peer from the "
                            <a href=routing::path::peers_overview>"peer overview page"</a>
                            "."
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
