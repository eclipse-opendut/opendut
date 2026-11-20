use leptos::prelude::*;

#[component]
pub fn DashboardCard<F>(
    #[prop(into)] title: String,
    #[prop(into)] href: String,
    footer: F,
    children: Children,
) -> impl IntoView
where F: IntoView + 'static {

    view! {
        <div class="card">
            <div class="card-header">
                <a class="card-header-title has-text-link" href=href>{title}</a>
            </div>
            <div class="card-content">
                <div class="level">
                    { children() }
                </div>
            </div>
            <div class="card-footer">
                <div class="m-2">
                    { footer }
                </div>
            </div>
        </div>
    }
}
