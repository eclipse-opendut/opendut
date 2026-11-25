use std::ops::Not;
use leptos::prelude::*;
use opendut_auth::types::Claims;
use crate::user::components::user_table::UserTableView;

const DEFAULT_KEYCLOAK_ROLES: [&str; 4] = [
    "offline_access",
    "uma_authorization",
    "managerrole",
    "testrole",
];

#[component]
pub fn PresentUserTableView(
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
