use leptos::either::EitherOf3;
use leptos::prelude::*;
use crate::user::{UserAuthentication, UserAuthenticationSignal};
use crate::user::components::UserTableView;

#[component]
pub fn UserOverview() -> impl IntoView {
    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");

    { move || { 
        match user.get() {
            UserAuthentication::Loading | UserAuthentication::Disabled | UserAuthentication::Unauthenticated => {
                EitherOf3::A(view!{ <UserTableView /> })
            }
            UserAuthentication::Authenticated(data) => {
                match data.token {
                    None => {
                        EitherOf3::B(view! { <UserTableView /> })
                    }
                    Some(auth_data) => {
                        let claims = auth_data.claims;

                        EitherOf3::C(view! {
                            <UserTableView claims />
                        })
                    }
                }
            }
        }
    }}
}
