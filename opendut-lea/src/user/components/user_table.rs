use std::ops::Not;
use leptos::prelude::*;
use opendut_auth::types::Claims;
use opendut_lea_components::{BasePageContainer, Breadcrumb, Divider, FontAwesomeIcon, IconText, IconTextDistance, ProfilePicture, ProfilePictureSize, Tag};
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
            <div class="columns is-mobile m-0">
                <div class="column is-narrow">
                    <ProfilePicture
                        name=Clone::clone(&name)
                        size=ProfilePictureSize::Large
                    />
                </div>
                <div class="column is-flex is-flex-direction-column is-justify-content-center">
                    <p class="title is-4"> { name } </p>
                    <p class="subtitle is-6 has-text-grey"> { preferred_username } </p>
                </div>
            </div>
            <Divider />
            <table class="table is-align-self-flex-start">
                <tbody>
                    <tr>
                        <td class="has-text-grey is-narrow">
                            <div class="pl-4 py-2">
                                <IconText
                                    icon=FontAwesomeIcon::Email
                                    text="Email"
                                    distance=IconTextDistance::Wide
                                />
                            </div>
                        </td>
                        <td>
                            <div class="py-2">
                                { email }
                            </div>
                        </td>
                    </tr>
                    <tr>
                        <td class="has-text-grey">
                            <div class="pl-4 py-2">
                                <IconText
                                    icon=FontAwesomeIcon::Shield
                                    text="Roles"
                                    distance=IconTextDistance::Wide
                                />
                            </div>
                        </td>
                        <td>
                            <div class="py-2">
                                {
                                    if roles.is_empty() {
                                        view! ("-").into_any()
                                    } else { ().into_any() }
                                }
                                <For
                                    each=move || roles.clone()
                                    key=|role| role.to_string()
                                    children=move |role| {
                                        view! {
                                            <span class="pr-3">
                                                <Tag text=role />
                                            </span>
                                        }
                                    }
                                />
                            </div>
                        </td>
                    </tr>
                    <tr>
                        <td class="has-text-grey">
                            <div class="pl-4 py-2">
                                <IconText
                                    icon=FontAwesomeIcon::Group
                                    text="Groups"
                                    distance=IconTextDistance::Wide
                                />
                            </div>
                        </td>
                        <td>
                            <div class="py-2">
                                {
                                    if groups.is_empty() {
                                        view! ("-").into_any()
                                    } else { ().into_any() }
                                }
                                <For
                                    each=move || groups.clone()
                                    key=|group| group.to_string()
                                    children=move |group| {
                                        view! {
                                            <span class="pr-3">
                                                <Tag text=group />
                                            </span>
                                        }
                                    }
                                />
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </BasePageContainer>
    }
}
