use leptos::prelude::*;
use opendut_types::proto::util::VersionInfo;
use crate::components::{GenerateSetupStringForm, GenerateSetupStringKind};

#[component]
pub fn CleoCard(
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
        format!("opendut-cleo-aarch64-unknown-linux-gnu{}.tar.gz", version_name())
    };
    let output_name_armv7 = move || {
        format!("opendut-cleo-armv7-unknown-linux-gnueabihf{}.tar.gz", version_name())
    };
    let output_name_x86 = move || {
        format!("opendut-cleo-x86_64-unknown-linux-gnu{}.tar.gz", version_name())
    };

    view! {
        <div class="card">
            <div class="card-header">
                <div class="card-header-title"><i class="fa-solid fa-terminal mr-1"></i>"CLEO"</div>
            </div>
            <div class="card-content">
            "Download for different architectures:"
                <div class="mb-2 mt-2 ml-2">
                    <a href="/api/cleo/aarch64-unknown-linux-gnu/download" download=output_name_aarch64>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"aarch64-gnu"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/cleo/armv7-unknown-linux-gnueabihf/download" download=output_name_armv7>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"armv7-gnueabihf"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/cleo/x86_64-unknown-linux-gnu/download" download=output_name_x86>
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
