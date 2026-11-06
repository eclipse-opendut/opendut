use std::sync::Arc;
use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::viper::{ViperSourceId, ViperTestId, ViperTestName};
use crate::app::use_app_globals;

#[component]
pub fn DeleteViperSourceButton<F>(
    #[prop(into)] viper_source_id: Signal<ViperSourceId>,
    #[prop(into)] button_color: Signal<ButtonColor>,
    on_delete: F
) -> impl IntoView
where F: Fn() + Clone + Send + 'static {

    let globals = use_app_globals();
    let pending = RwSignal::new(true);
    let toaster = use_toaster();

    let used_by: LocalResource<Result<Vec<(ViperTestId, ViperTestName)>, String>> = {
        let carl = globals.client.clone();
        let toaster = toaster.clone();
        LocalResource::new(move || {
            let source_id = viper_source_id.get();
            let mut carl = carl.clone();
            let toaster = toaster.clone();
            pending.set(true);

            async move {
                let result = carl.viper.list_viper_test_run_descriptors().await
                    .map(|tests| tests.into_iter()
                        .filter(|test| test.source == source_id)
                        .map(|test| (test.id, test.name))
                        .collect())
                    .map_err(|cause| {
                        error!("Failed to load tests using VIPER source <{source_id}>, due to error: {cause}");
                        toaster.toast(Toast::builder()
                            .simple("Failed to check VIPER source usage.")
                            .error());
                        cause.to_string()
                    });
                pending.set(false);
                result
            }
        })
    };

    let button_state = Signal::derive(move || {
        if pending.get() {
            ButtonState::Loading
        } else {
            match used_by.get() {
                None => ButtonState::Loading,
                Some(Ok(tests)) if tests.is_empty() => ButtonState::Enabled,
                Some(_) => ButtonState::Disabled,
            }
        }
    });

    let on_confirm = move || {
        let on_delete = on_delete.clone();
        let mut carl = globals.client.clone();
        let id = viper_source_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.viper.delete_viper_source_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted viper source: {}", viper_source_id.get_untracked());
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted viper source successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to delete viper source <{}>, due to error: {cause}", viper_source_id.get_untracked());
                    toaster.toast(
                        Toast::builder()
                            .simple(cause.to_string())
                            .error()
                    );
                }
            }

            pending.set(false);
        });
    };

    let hide_tooltip = Signal::derive(move || {
        !matches!(button_state.get(), ButtonState::Disabled)
    });

    let tooltip_content = Arc::new(move || {
        match used_by.get() {
            Some(Ok(tests)) => view! {
                <div class="content has-text-left">
                    <p>VIPER source can not be deleted while it is used by a VIPER test.</p>
                    <p>"Usage:"</p>
                    <ul>
                        {tests.into_iter().map(|(id, name)| view! {
                            <li title=id.to_string()>
                                <a href=format!("/viper_tests/{id}/configure/general")> {name.to_string()} </a>
                            </li>
                        }).collect_view()}
                    </ul>
                </div>
            }.into_any(),
            Some(Err(_)) => view! {
                <p>"VIPER source usage could not be checked."</p>
            }.into_any(),
            None => ().into_any(),
        }
    });

    view! {
        <Tooltip
            text=tooltip_content
            direction=TooltipDirection::Right
            is_hidden=hide_tooltip
        >
            <ConfirmationButton
                icon=FontAwesomeIcon::TrashCan
                color=button_color
                size=ButtonSize::Normal
                state=button_state
                label="Remove Viper Source?"
                on_confirm
            />
        </Tooltip>
    }
}
