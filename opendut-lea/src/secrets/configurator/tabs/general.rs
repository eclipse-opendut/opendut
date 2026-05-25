use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::secrets::configurator::components::{SecretNameInput, SecretValueInput};
use crate::secrets::configurator::types::UserSecretConfiguration;

#[component]
pub fn GeneralTab(secret_configuration: RwSignal<UserSecretConfiguration>) -> impl IntoView {

    let secret_id = Signal::derive(move || secret_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Secret ID"
                value=secret_id
            />
            <SecretNameInput
                secret_configuration
            />
            <SecretValueInput
                secret_configuration
            />
        </div>
    }
}
