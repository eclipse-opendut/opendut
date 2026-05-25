use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, OverviewTableCell};
use opendut_model::secret::{SecretDescriptor, SecretValue};
use crate::components::ClickableOverviewTableRow;
use crate::secrets::components::DeleteSecretButton;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    secret_descriptor: RwSignal<SecretDescriptor>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where OnDeleteFn: Fn() + Copy + Send + 'static, {

    let secret_id = create_read_slice(secret_descriptor,
        |secret_descriptor| {
            secret_descriptor.id
        }
    );

    let secret_name = create_read_slice(secret_descriptor,
        |secret_descriptor| {
            secret_descriptor.name.to_string()
        }
    );

    let secret_type = create_read_slice(secret_descriptor,
        |secret_descriptor| {
            match &secret_descriptor.value {
                SecretValue::Token(_) => String::from("Token"),
            }
        }
    );

    let configurator_href = Signal::derive(move || { format!("/secrets/{}/configure/general", secret_id.get()) });

    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <a href=configurator_href> { secret_name } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                { secret_type }
            </OverviewTableCell>

            <OverviewTableCell>
                <div class="is-pulled-right">
                    <DeleteSecretButton
                        secret_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
