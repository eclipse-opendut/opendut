use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb};

#[component]
pub fn UserTableView(
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