use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, OverviewTableCell};
use opendut_model::viper::ViperTestDescriptor;
use crate::app::use_app_globals;
use crate::components::ClickableOverviewTableRow;
use crate::tests::components::DeleteTestButton;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    test_descriptor: RwSignal<ViperTestDescriptor>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where OnDeleteFn: Fn() + Copy + Send + 'static, {

    let globals = use_app_globals();

    let test_id = create_read_slice(test_descriptor,
        |test_descriptor| {
            test_descriptor.id
        }
    );

    let test_name = create_read_slice(test_descriptor,
        |test_descriptor| {
            test_descriptor.name.to_string()
        }
    );

    let test_source_id = create_read_slice(test_descriptor,
        |test_descriptor| {
            test_descriptor.source
        }
    );

    let test_source_descriptor = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        let source_id = test_source_id.get();

        async move {
            carl.viper.get_viper_source_descriptor(source_id).await
                .expect("Failed to request the viper source by id.")
        }
    });

    let test_source = Signal::derive(move || {
        test_source_descriptor.read().as_ref()
            .map(|descriptor| descriptor.name.to_string())
    });

    let test_suite = create_read_slice(test_descriptor,
        |test_descriptor| {
            test_descriptor.suite.to_string()
        }
    );

    let configurator_href = Signal::derive(move || { format!("/tests/{}/configure/general", test_id.get()) });
    let source_configurator_href = move || { format!("/sources/{}/configure/general", test_source_id.get()) };

    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <a href=configurator_href> { test_name } </a>
            </OverviewTableCell>
        
            { move ||
                test_source.get().map(|source_name| {
                    view! {
                        <OverviewTableCell>
                            <a href=source_configurator_href on:click=move |ev| ev.stop_propagation()>
                                { source_name }
                            </a>
                        </OverviewTableCell>
                    }
                })
            }
        
            <OverviewTableCell>
                <p> { test_suite } </p>
            </OverviewTableCell>
        
            <OverviewTableCell>
                <div class="is-pulled-right">
                    <DeleteTestButton
                        test_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
