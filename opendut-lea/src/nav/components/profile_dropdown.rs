use leptos::html::Div;
use leptos::prelude::*;
use leptos_oidc::LogoutLink;
use leptos_use::on_click_outside;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, Doorhanger, DoorhangerAlignment, FontAwesomeIcon, Hsl, IconButton, IconText, ProfilePictureColors, ToggleSignal};
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

    let initials = Signal::derive(move || {
        fullname.get()
            .split_whitespace()
            .filter_map(|word| word.get(0..1))
            .take(2)
            .collect::<String>()
            .to_uppercase()
    });
    let hsl = Signal::derive(move || {
        let hsl_colors = ProfilePictureColors::get_vec();
        let sum: u32 = fullname.get().chars().map(|c| c as u32).sum();
        let index = (sum % hsl_colors.len() as u32) as usize;
        hsl_colors[index].get_hsl()
    });

    view! {
        <div node_ref=profile_button_area>
            <Doorhanger
                visible=dropdown_visible.read_only()
                alignment=DoorhangerAlignment::Left
                trigger=Box::new({
                    let initials = Clone::clone(&initials);
                    move || {
                        view! {
                            <button class="button dut-profile-picture-button is-text p-0" on:click=move |_| dropdown_visible.toggle()>
                                <ProfilePicture initials size=ProfilePictureSize::Small hsl />
                            </button>
                        }.into_any()
                    }
                })
                show_dog_ear=false
            >
                <div class="columns m-0">
                    <div class="column is-narrow">
                        <ProfilePicture initials size=ProfilePictureSize::Large hsl />
                    </div>
                    <div class="column">
                        <div class="is-flex pb-2">
                            <p class="dut-fullname-text pr-2"> { fullname }</p>
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

enum ProfilePictureSize {
    Small,
    Large
}

impl ProfilePictureSize {
    pub fn get_size(&self) -> u32 {
        match self {
            ProfilePictureSize::Small => 38,
            ProfilePictureSize::Large => 64
        }
    }
}

#[component]
fn ProfilePicture(
    #[prop(into)] initials: Signal<String>,
    size: ProfilePictureSize,
    #[prop(into)] hsl: Signal<Hsl>,
) -> impl IntoView {

    let h = move || hsl.read().0;
    let s = move || hsl.read().1;
    let l = move || hsl.read().2;

    let text_color = move || {
        format!("hsl({},{}%,{}%)", h(), s(), l())
    };
    let background_color = move || format!("hsla({},{}%,{}%, 0.1)", h(), s(), l());

    let size = size.get_size();

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width=size height=size viewBox="0 0 64 64">
            <circle cx="32" cy="32" r="32" fill= move || background_color />
            <text x="50%" y="50%" text-anchor="middle" dominant-baseline="central" font-size="24" fill= move || text_color font-weight="500">
                { initials }
            </text>
        </svg>
    }
}
