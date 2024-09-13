use leptos::*;
use leptos_router::use_navigate;
use tracing::info;
use url::Url;

use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
pub use routes::AppRoutes as Routes;

use crate::components::BasePageContainer;
use crate::util::url::UrlEncodable;

pub mod path {
    #![allow(non_upper_case_globals)]

    pub const dashboard: &str = "/";

    pub const about: &str = "/about";
    pub const downloads: &str = "/downloads";
    pub const clusters_overview: &str = "/clusters";
    pub const error: &str = "/error";
    pub const licenses: &str = "/licenses";
    pub const peers_overview: &str = "/peers";
    pub const user: &str = "/user";
}

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
                base.join(path::clusters_overview)
                    .expect("ClustersOverview route should be valid.")
            },
            WellKnownRoutes::ClusterConfigurator { id } => {
                base.join(&format!("/clusters/{}/configure/general", id.url_encode()))
                    .expect("ClusterConfigurator route should be valid.")
            },
            WellKnownRoutes::PeersOverview => {
                base.join(path::peers_overview)
                    .expect("PeerOverview route should be valid.")
            },
            WellKnownRoutes::PeerConfigurator { id } => {
                base.join(&format!("/peers/{}/configure/general", id.url_encode()))
                    .expect("PeerConfigurator route should be valid.")
            },
            WellKnownRoutes::ErrorPage { title, text, details } => {
                let mut url = base.join(path::error).unwrap();
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
    use crate::routing::{self, NotFound};
    use crate::user::UserOverview;
    use crate::about::AboutOverview;
    use crate::downloads::Downloads;

    #[component]
    pub fn AppRoutes() -> impl IntoView {
        view! {
            <Router>
                <main>
                    <Routes>
                        <Route path=routing::path::dashboard view=|| view! { <Dashboard /> } />
                        <Route path=routing::path::clusters_overview view=|| view! { <ClustersOverview /> } />
                        <Route path="/clusters/:id/configure/:tab" view=|| view! { <ClusterConfigurator /> } />
                        <Route path=routing::path::peers_overview view=|| view! { <PeersOverview /> } />
                        <Route path="/peers/:id/configure/:tab" view=|| view! { <PeerConfigurator /> } />
                        <Route path=routing::path::downloads view=|| view! { <Downloads /> } />
                        <Route path=routing::path::user view=|| view! { <UserOverview /> } />
                        <Route path=routing::path::licenses view=|| view! { <LicensesOverview /> } />
                        <Route path=routing::path::about view=|| view! { <AboutOverview /> } />
                        <Route path=routing::path::error view=|| view! { <ErrorPage /> } />
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
