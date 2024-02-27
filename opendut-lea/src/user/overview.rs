use std::ops::Not;
use leptos::*;
use opendut_carl_api::carl::wasm::OptionalAuthData;
use crate::components::{BasePageContainer, Breadcrumb, Initialized};

#[component]
pub fn UserOverview() -> impl IntoView {

    fn Inner() -> impl IntoView {

        const DEFAULT_KEYCLOAK_ROLES: [&'static str; 4] = [
            "offline_access",
            "uma_authorization",
            "managerrole",
            "testrole",
        ];

        let (auth_data_signal, _) = use_context::<(ReadSignal<OptionalAuthData>, WriteSignal<OptionalAuthData>)>().expect("AuthData should be provided in the context.");

        match auth_data_signal.get().auth_data {
            None => {
                view! {
                    <CreateTableView
                        preferred_username="UNKNOWN USER"
                        name=""
                        email=""
                        groups=""
                        roles=""
                    >
                    </CreateTableView>
                }
            }
            Some(auth_data) => {
                let user_name  = auth_data.preferred_username;
                let name  = auth_data.name;
                let email  = auth_data.email;
                let roles  = auth_data.roles.into_iter().filter(| role | {
                        DEFAULT_KEYCLOAK_ROLES.contains(&role.as_str()).not()
                    }).collect::<Vec<_>>().join(", ");
                let groups  = auth_data.groups.into_iter().map(| group | {
                        group.replace('/', "")
                    }).collect::<Vec<_>>().join(", ");

                view! {
                    <CreateTableView
                        preferred_username=user_name
                        name=name
                        email=email
                        groups=groups
                        roles=roles
                    >
                    </CreateTableView>
                }
            }
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}

#[component]
fn CreateTableView(
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