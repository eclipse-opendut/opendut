use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use opendut_lea_components::ButtonColor;
use opendut_model::viper::ViperRunDescriptor;
use crate::app::use_app_globals;
use crate::tests::components::DeleteTestButton;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    test_descriptor: RwSignal<ViperRunDescriptor>,
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
            test_descriptor.id.to_string() // Todo: Replace by name
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
            .map(|descriptor| (descriptor.name.to_string(), descriptor.url.to_string()))
    });

    let test_suite = create_read_slice(test_descriptor,
        |test_descriptor| {
            test_descriptor.suite.to_string()
        }
    );

    let configurator_href = move || { format!("/tests/{}/configure/general", test_id.get()) };

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    view! {
        <tr>
            <td class="is-vcentered">
                <a href=configurator_href> { test_name } </a>
            </td>
            { move ||
                test_source.get().map(|(source_name, source_url)| {
                    view! {
                        <td class="is-vcentered">
                            <a href=source_url> { source_name } </a>
                        </td>
                    }
                })
            }
            <td class="is-vcentered">
                <p> { test_suite } </p>
            </td>
            <td class="is-vcentered is-flex is-justify-content-center">
                <div class="is-pulled-right">
                    <DeleteTestButton
                        test_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </td>
        </tr>
    }
}
