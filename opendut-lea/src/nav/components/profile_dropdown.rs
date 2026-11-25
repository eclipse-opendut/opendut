use leptos::html::Div;
use leptos::prelude::*;
use leptos_oidc::LogoutLink;
use leptos_use::on_click_outside;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, Doorhanger, DoorhangerAlignment, FontAwesomeIcon, IconButton, IconText, ProfilePicture, ProfilePictureSize, ToggleSignal};
use crate::routing;
use crate::user::UserAuthenticationSignal;

#[component]
pub fn ProfileDropdown() -> impl IntoView {

    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");
    let fullname = Signal::derive(move || {
       user.read().fullname().unwrap_or_else(|| String::from("Unknown User"))
    });

    let username = Signal::derive(move || {
        user.read().username()
    });
    let email = Signal::derive(move || {
        user.read().email().unwrap_or_else(|| String::from("no email"))
    });

    let dropdown_visible = RwSignal::new(false);
    let profile_button_area = NodeRef::<Div>::new();
    let _ = on_click_outside(profile_button_area, move |_| {
        dropdown_visible.set(false)
    });

    view! {
        <div node_ref=profile_button_area>
            <Doorhanger
                visible=dropdown_visible.read_only()
                alignment=DoorhangerAlignment::Left
                trigger=Box::new({
                    move || {
                        view! {
                            <button class="button dut-profile-picture-button is-text p-0" on:click=move |_| dropdown_visible.toggle()>
                                <ProfilePicture name=fullname size=ProfilePictureSize::Small />
                            </button>
                        }.into_any()
                    }
                })
                show_dog_ear=false
            >
                <div class="columns m-0">
                    <div class="column is-narrow">
                        <ProfilePicture name=fullname size=ProfilePictureSize::Large />
                    </div>
                    <div class="column">
                        <div class="is-flex pb-2">
                            <p class="dut-fullname-text pr-2"> { fullname } </p>
                            <p class="dut-username-text"> { username } </p>
                        </div>
                        <div class="dut-profile-information">
                            <IconText
                                icon=FontAwesomeIcon::Email
                                text=email
                            />
                            <IconText
                                icon=FontAwesomeIcon::OpenPage
                                text="My Profile"
                                href=routing::path::user
                            />
                        </div>
                    </div>
                </div>
                <LogoutLink class="is-flex is-justify-content-end p-1">
                    <IconButton
                        icon=FontAwesomeIcon::SignOut
                        color=ButtonColor::White
                        size=ButtonSize::Normal
                        state=ButtonState::default()
                        label="Log out"
                        show_label=true
                        on_action=move || {}
                    />
                </LogoutLink>
            </Doorhanger>
        </div>
    }
}
