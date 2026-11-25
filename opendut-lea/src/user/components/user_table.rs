use std::ops::Not;
use leptos::prelude::*;
use opendut_auth::types::Claims;
use opendut_lea_components::{BasePageContainer, Breadcrumb, Divider, FontAwesomeIcon, IconText, IconTextDistance, ProfilePicture, ProfilePictureSize, TagFromList};
use crate::user::DEFAULT_KEYCLOAK_ROLES;

struct UserData {
    preferred_username: String,
    name: String,
    email: String,
    roles: Vec<String>,
    groups: Vec<String>,
}

#[component]
pub fn UserTableView(
    #[prop(into, optional)] claims: Option<Claims>,
) -> impl IntoView {

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("User Profile", "/user")
    ];

    let data = match claims {
        Some(claims) => {
            let roles = claims.additional_claims.roles.into_iter().filter(| role | {
                DEFAULT_KEYCLOAK_ROLES.contains(&role.as_str()).not()
            }).collect::<Vec<_>>();

            let groups  = claims.additional_claims.groups.into_iter().map(| group | {
                group.replace('/', "")
            }).collect::<Vec<_>>();

            UserData {
                preferred_username: claims.preferred_username,
                name: claims.name,
                email: claims.email,
                roles,
                groups,
            }
        }
        None => {
            UserData {
                preferred_username: String::from("disabled"),
                name: String::from("Unknown User"),
                email: String::from("-"),
                roles: Vec::new(),
                groups: Vec::new(),
            }
        }
    };

    let UserData { preferred_username, name, email, roles, groups} = data;

    view! {
        <BasePageContainer
            title="User Profile"
            breadcrumbs=breadcrumbs
            controls=view! { <> }
        >
            <div class="is-inline-block">
                <div class="columns is-mobile m-0 pr-5">
                    <div class="column is-narrow">
                        <ProfilePicture
                            name=Clone::clone(&name)
                            size=ProfilePictureSize::Large
                        />
                    </div>
                    <div class="column is-flex is-flex-direction-column is-justify-content-center">
                        <p class="title is-4 mb-0"> { name } </p>
                        <p class="has-text-grey">
                            <IconText
                                icon=FontAwesomeIcon::At
                                text=preferred_username
                            />
                        </p>
                    </div>
                </div>
                <Divider />
                <table class="table is-fullwidth">
                    <tbody>
                        <UserTableRow icon=FontAwesomeIcon::Email label="Email">
                            { email }
                        </UserTableRow>
                        <UserTableRow icon=FontAwesomeIcon::Shield label="Roles">
                            <TagFromList items=roles />
                        </UserTableRow>
                        <UserTableRow icon=FontAwesomeIcon::Group label="Groups">
                            <TagFromList items=groups />
                        </UserTableRow>
                    </tbody>
                </table>
            </div>
        </BasePageContainer>
    }
}

#[component]
fn UserTableRow(
    icon: FontAwesomeIcon,
    label: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <tr>
            <td class="has-text-grey">
                <div class="px-4 py-2">
                    <IconText
                        icon
                        text=label
                        distance=IconTextDistance::Wide
                    />
                </div>
            </td>
            <td>
                <div class="py-2">{ children() }</div>
            </td>
        </tr>
    }
}
