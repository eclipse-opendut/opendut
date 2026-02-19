use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, OverviewTableCell};
use opendut_model::viper::ViperSourceDescriptor;
use crate::components::ClickableOverviewTableRow;
use crate::sources::components::DeleteSourceButton;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    source_descriptor: RwSignal<ViperSourceDescriptor>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where OnDeleteFn: Fn() + Copy + Send + 'static, {

    let source_id = create_read_slice(source_descriptor,
        |source_descriptor| {
            source_descriptor.id
        }
    );

    let source_name = create_read_slice(source_descriptor,
        |source_descriptor| {
            source_descriptor.name.to_string()
        }
    );

    let source_url = create_read_slice(source_descriptor,
        |source_descriptor| {
            source_descriptor.url.to_string()
        }
    );

    let configurator_href = Signal::derive(move || { format!("/sources/{}/configure/general", source_id.get()) });
    
    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <a href=configurator_href> { source_name } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                <a href=source_url> { source_url } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                <div class="is-pulled-right">
                    <DeleteSourceButton
                        source_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
