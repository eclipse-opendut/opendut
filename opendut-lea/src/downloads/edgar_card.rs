use leptos::prelude::*;
use opendut_model::proto::util::VersionInfo;
use crate::routing;

#[component]
pub fn EdgarCard(
    version_info: LocalResource<VersionInfo>
) -> impl IntoView {
    
    let version_name = move || {
        match version_info.get() {
            None => { String::new() }
            Some(version) => {
                format!("-{}", version.name)
            }
        }
    };

    let output_name_aarch64 = move || {
        format!("opendut-edgar-aarch64-unknown-linux-gnu{}.tar.gz", version_name())
    };
    let output_name_armv7 = move || {
        format!("opendut-edgar-armv7-unknown-linux-gnueabihf{}.tar.gz", version_name())
    };
    let output_name_x86 = move || {
        format!("opendut-edgar-x86_64-unknown-linux-gnu{}.tar.gz", version_name())
    };

    view! {
        <div class="card">
            <div class="card-header">
                <div class="card-header-title"><i class="fa-solid fa-microchip mr-1"></i>"EDGAR"</div>
            </div>
            <div class="card-content">
                "Download for different architectures:"
                <div class="mt-2 mb-2 ml-2">
                    <a href="/api/edgar/aarch64-unknown-linux-gnu/download" download=output_name_aarch64>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"aarch64-gnu"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/edgar/armv7-unknown-linux-gnueabihf/download" download=output_name_armv7>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"armv7-gnueabihf"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/edgar/x86_64-unknown-linux-gnu/download" download=output_name_x86>
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
