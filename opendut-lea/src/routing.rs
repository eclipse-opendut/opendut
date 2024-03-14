use leptos::*;
use leptos_router::use_navigate;
use tracing::info;
use url::Url;

use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
pub use routes::AppRoutes as Routes;

use crate::components::BasePageContainer;
use crate::util::url::UrlEncodable;

pub enum WellKnownRoutes {
    ClustersOverview,
    ClusterConfigurator { id: ClusterId },
    PeerConfigurator { id: PeerId },
    PeersOverview,
    ErrorPage { title: String, text: String, details: Option<String> },
}

impl WellKnownRoutes {

    fn route(&self, base: &Url) -> Url {
        match self {
            WellKnownRoutes::ClustersOverview => {
                base.join("/clusters")
                    .expect("ClustersOverview route should be valid.")
            },
            WellKnownRoutes::ClusterConfigurator { id } => {
                base.join(&format!("/clusters/{}/configure/general", id.url_encode()))
                    .expect("ClusterConfigurator route should be valid.")
            },
            WellKnownRoutes::PeersOverview => {
                base.join("/peers")
                    .expect("PeerOverview route should be valid.")
            },
            WellKnownRoutes::PeerConfigurator { id } => {
                base.join(&format!("/peers/{}/configure/general", id.url_encode()))
                    .expect("PeerConfigurator route should be valid.")
            },
            WellKnownRoutes::ErrorPage { title, text, details } => {
                let mut url = base.join("/error").unwrap();
                {
                    let mut query = url.query_pairs_mut();
                    query.append_pair("title", title);
                    query.append_pair("text", text);
                    if let Some(details) = details {
                        query.append_pair("details", details);
                    }
                }
                url
            }
        }
    }
}

mod routes {
    use leptos::*;
    use leptos_router::{Route, Router, Routes};

    use crate::clusters::{ClusterConfigurator, ClustersOverview};
    use crate::dashboard::Dashboard;
    use crate::error::ErrorPage;
    use crate::licenses::LicensesOverview;
    use crate::peers::{PeerConfigurator, PeersOverview};
    use crate::routing::NotFound;
    use crate::user::UserOverview;
    use crate::about::AboutOverview;

    #[component]
    pub fn AppRoutes() -> impl IntoView {
        view! {
            <Router>
                <main>
                    <Routes>
                        <Route path="/" view=|| view! { <Dashboard /> } />
                        <Route path="/clusters" view=|| view! { <ClustersOverview /> } />
                        <Route path="/clusters/:id/configure/:tab" view=|| view! { <ClusterConfigurator /> } />
                        <Route path="/peers" view=|| view! { <PeersOverview /> } />
                        <Route path="/peers/:id/configure/:tab" view=|| view! { <PeerConfigurator /> } />
                        <Route path="/user" view=|| view! { <UserOverview /> } />
                        <Route path="/licenses" view=|| view! { <LicensesOverview /> } />
                        <Route path="/about" view=|| view! { <AboutOverview /> } />
                        <Route path="/error" view=|| view! { <ErrorPage /> } />
                        <Route path="/*any" view=|| view! { <NotFound /> } />
                    </Routes>
                </main>
            </Router>
        }
    }
}

pub fn navigate_to(route: WellKnownRoutes) {

    let base = {
        let location = leptos_dom::helpers::location();
        Url::parse(location.origin()
            .expect("Origin of the current location should be valid.").as_str())
            .expect("Base url should be valid.")
    };

    let route = {
        let url = route.route(&base);
        let mut result = String::from(url.path());
        if let Some(query) = url.query() {
            result.push('?');
            result.push_str(query);
        }
        result
    };

    info!("Navigating to {}", route);

    let navigate = use_navigate();

    request_animation_frame(move || {
        navigate(&route, Default::default());
    });
}

#[component]
fn NotFound() -> impl IntoView {

    view! {

        <BasePageContainer
            title="Not Found"
            breadcrumbs=Vec::new()
            controls=|| ()
        >
            <p class="subtitle">"The page you are looking for does not exist."</p>
        </BasePageContainer>
    }
}
