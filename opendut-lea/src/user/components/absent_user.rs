use leptos::prelude::*;
use crate::user::components::user_table::UserTableView;

#[component]
pub fn AbsentUserTableView() -> impl IntoView {
    view! {
        <UserTableView
            preferred_username=""
            name=""
            email=""
            groups=""
            roles=""
        >
        </UserTableView>
    }
}
