use leptos::either::EitherOf3;
use leptos::prelude::*;
use crate::user::{UserAuthentication, UserAuthenticationSignal};
use crate::user::components::{AbsentUserTableView, PresentUserTableView};

#[component]
pub fn UserOverview() -> impl IntoView {
    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");

    { move || { 
        match user.get() {
            UserAuthentication::Loading | UserAuthentication::Disabled | UserAuthentication::Unauthenticated => {
                EitherOf3::A(view!{ <AbsentUserTableView /> })
            }
            UserAuthentication::Authenticated(data) => {
                match data.token {
                    None => {
                        EitherOf3::B(view! { <AbsentUserTableView /> })
                    }
                    Some(auth_data) => {
                        let claims = auth_data.claims;
                        EitherOf3::C(view! { <PresentUserTableView claims /> })
                    }
                }
            }
        }
    }}
}
