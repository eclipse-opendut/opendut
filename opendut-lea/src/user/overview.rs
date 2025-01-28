use std::ops::Not;
use leptos::either::EitherOf3;
use leptos::prelude::*;
use opendut_auth::types::Claims;
use crate::components::{BasePageContainer, Breadcrumb};
use crate::user::{UserAuthentication, UserAuthenticationSignal, UNAUTHENTICATED_USER};

const DEFAULT_KEYCLOAK_ROLES: [&str; 4] = [
    "offline_access",
    "uma_authorization",
    "managerrole",
    "testrole",
];

#[component]
pub fn UserOverview() -> impl IntoView {
    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");

    { move || { 
        match user.get() {
            UserAuthentication::Loading | UserAuthentication::Disabled | UserAuthentication::Unauthenticated => {
                EitherOf3::A(view!{ <AbsentUserTableView/> })
            }
            UserAuthentication::Authenticated(token) => {
                match token {
                    None => {
                        EitherOf3::B(view! { <AbsentUserTableView/> })
                    }
                    Some(auth_data) => {
                        let claims = auth_data.claims;
                        EitherOf3::C(view! { <PresentUserTableView claims/> })
                    }
                }
            }
        }
    }}
}

#[component]
fn AbsentUserTableView() -> impl IntoView {
    
    view! { 
        <UserTableView
            preferred_username=UNAUTHENTICATED_USER
            name=""
            email=""
            groups=""
            roles=""
        >
        </UserTableView>
    }
}

#[component]
fn PresentUserTableView(
    claims: Claims
) -> impl IntoView {

    let user_name = claims.preferred_username;
    let name = claims.name;
    let email = claims.email;
    let roles = claims.additional_claims.roles.into_iter().filter(| role | {
        DEFAULT_KEYCLOAK_ROLES.contains(&role.as_str()).not()
    }).collect::<Vec<_>>().join(", ");
    let groups  = claims.additional_claims.groups.into_iter().map(| group | {
        group.replace('/', "")
    }).collect::<Vec<_>>().join(", ");

    view! {
        <UserTableView
            preferred_username=user_name
            name=name
            email=email
            groups=groups
            roles=roles
        >
        </UserTableView>
    }

}


#[component]
fn UserTableView(
    #[prop(into)] preferred_username: String,
    #[prop(into)] name: String,
    #[prop(into)] email: String,
    #[prop(into)] groups: String,
    #[prop(into)] roles: String,
) -> impl IntoView {
    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("User Profile", "/user")
    ];

    view! {
        <BasePageContainer
            title="User Profile"
            breadcrumbs=breadcrumbs
            controls=view! { <div></div> }
        >
            <table class="table is-stripped">
                <tbody>
                    <tr>
                        <td>Username</td> <td>{ preferred_username }</td>
                    </tr>
                    <tr>
                        <td>Fullname</td> <td>{ name }</td>
                    </tr>
                    <tr>
                        <td>Email</td> <td>{ email }</td>
                    </tr>
                    <tr>
                        <td>Roles</td> <td>{ roles }</td>
                    </tr>
                    <tr>
                        <td>Groups</td> <td>{ groups }</td>
                    </tr>
                </tbody>
            </table>
        </BasePageContainer>
    }
}
