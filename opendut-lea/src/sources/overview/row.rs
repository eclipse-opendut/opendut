use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use opendut_lea_components::ButtonColor;
use opendut_model::viper::ViperSourceDescriptor;
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

    let configurator_href = move || { format!("/sources/{}/configure/general", source_id.get()) };

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    view! {
        <tr>
            <td class="is-vcentered">
                <a href=configurator_href> { source_name } </a>
            </td>
            <td class="is-vcentered">
                <a href=source_url> { source_url } </a>
            </td>
            <td class="is-vcentered is-flex is-justify-content-center">
                <div class="is-pulled-right">
                    <DeleteSourceButton
                        source_id
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </td>
        </tr>
    }
}
