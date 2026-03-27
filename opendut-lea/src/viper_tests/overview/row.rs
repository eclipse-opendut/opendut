use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, OverviewTableCell};
use opendut_model::viper::ViperTestRunDescriptor;
use crate::app::use_app_globals;
use crate::components::ClickableOverviewTableRow;
use crate::viper_tests::components::DeleteViperTestButton;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    viper_test_run_descriptor: RwSignal<ViperTestRunDescriptor>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where OnDeleteFn: Fn() + Copy + Send + 'static, {

    let globals = use_app_globals();

    let viper_test_id = create_read_slice(viper_test_run_descriptor,
        |viper_test_run_descriptor| {
            viper_test_run_descriptor.id
        }
    );

    let viper_test_name = create_read_slice(viper_test_run_descriptor,
        |viper_test_run_descriptor| {
            viper_test_run_descriptor.name.to_string()
        }
    );

    let viper_source_id = create_read_slice(viper_test_run_descriptor,
        |viper_test_run_descriptor| {
            viper_test_run_descriptor.source
        }
    );

    let viper_source_descriptor = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        let viper_source_id = viper_source_id.get();

        async move {
            carl.viper.get_viper_source_descriptor(viper_source_id).await
                .expect("Failed to request the viper source by id.")
        }
    });

    let configurator_href = Signal::derive(move || { format!("/viper_tests/{}/configure/general", viper_test_id.get()) });
    let viper_source_configurator_href = move || { format!("/viper_sources/{}/configure/general", viper_source_id.get()) };

    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <a href=configurator_href> { viper_test_name } </a>
            </OverviewTableCell>
        
            { move || Suspend::new(async move {
                let viper_source = viper_source_descriptor.await;
                let viper_source_name = viper_source.name.to_string();
                view! {
                    <OverviewTableCell>
                        <a href=viper_source_configurator_href>
                            { viper_source_name }
                        </a>
                    </OverviewTableCell>
                }
            })}
        
            <OverviewTableCell>
                <div class="is-pulled-right">
                    <DeleteViperTestButton
                        viper_test_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
