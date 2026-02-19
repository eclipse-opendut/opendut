use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, OverviewTableCell};
use opendut_model::viper::ViperSourceDescriptor;
use crate::components::ClickableOverviewTableRow;
use crate::viper_sources::components::DeleteViperSourceButton;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    viper_source_descriptor: RwSignal<ViperSourceDescriptor>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where OnDeleteFn: Fn() + Copy + Send + 'static, {

    let viper_source_id = create_read_slice(viper_source_descriptor,
        |viper_source_descriptor| {
            viper_source_descriptor.id
        }
    );

    let viper_source_name = create_read_slice(viper_source_descriptor,
        |viper_source_descriptor| {
            viper_source_descriptor.name.to_string()
        }
    );

    let viper_source_url = create_read_slice(viper_source_descriptor,
        |viper_source_descriptor| {
            viper_source_descriptor.url.to_string()
        }
    );

    let configurator_href = Signal::derive(move || { format!("/viper_sources/{}/configure/general", viper_source_id.get()) });
    
    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <a href=configurator_href> { viper_source_name } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                <a href=viper_source_url> { viper_source_url } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                <div class="is-pulled-right">
                    <DeleteViperSourceButton
                        viper_source_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
