use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, Doorhanger, DoorhangerAlignment, FontAwesomeIcon, IconButton, ProfilePictureColors};
use crate::user::UserAuthenticationSignal;

#[component]
pub fn ProfileDoorhanger() -> impl IntoView {

    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");

    let fullname = user.read().fullname();
    let username = user.read().username();
    let email = user.read().email();

    let fullname = Some(String::from("Vivian Berger"));
    let username = String::from("SaFend");
    let email = Some(String::from("salvatore.fendt@mercedes-benz.com"));

    // Effect::new(move || {
    //     fullname = user.read().fullname();
    //     username = user.read().username();
    //     email = user.read().email();
    // });

    let doorhanger_visible = RwSignal::new(false);
    let profile_button_area = NodeRef::<Div>::new();
    let _ = on_click_outside(profile_button_area, move |_| {
        doorhanger_visible.set(false)
    });

    view! {
        <div node_ref=profile_button_area>
            <Doorhanger
                visible=doorhanger_visible.read_only()
                alignment=DoorhangerAlignment::Left
                trigger=Box::new(move || {
                    view! {
                        <IconButton
                            icon=FontAwesomeIcon::User
                            color=ButtonColor::None
                            size=ButtonSize::Normal
                            state=ButtonState::default()
                            label="Open Profile"
                            on_action=move || doorhanger_visible.set(true)
                        />
                    }.into_any()
                })
            >
                <div class="columns m-0">
                    <div class="column">
                        <AlternativeProfilePicture fullname = fullname.clone().unwrap_or_default() />
                    </div>
                    <div class="column">
                        <div class="is-flex">
                            <p class="dut-fullname-text pr-2"> { fullname }</p>
                            <p class="dut-username-text"> { username } </p>
                        </div>
                        <div class="is-flex is-align-content-center">
                            <span class="icon pr-2">
                                <i class=FontAwesomeIcon::Email.as_class() />
                            </span>
                            <span class="dut-email-text"> { email } </span>
                        </div>
                    </div>
                </div>
                <div class="is-flex is-justify-content-end">
                    <IconButton
                        icon=FontAwesomeIcon::SignOut
                        color=ButtonColor::White
                        size=ButtonSize::Normal
                        state=ButtonState::default()
                        label="Log out"
                        show_label=true
                        on_action=move || {}
                    />
                </div>
            </Doorhanger>
        </div>
    }
}

#[component]
fn AlternativeProfilePicture(
    #[prop(into)] fullname: String
) -> impl IntoView {

    let hsl_colors = ProfilePictureColors::get_vec();

    let sum: u32 = fullname.chars().map(|c| c as u32).sum();
    let index = (sum % hsl_colors.len() as u32) as usize;
    let hsl = hsl_colors[index].get_hsl();
    let h = hsl.0;
    let s = hsl.1;
    let l = hsl.2;

    let text_color = format!("hsl({h},{s}%,{l}%)");
    let background_color = format!("hsla({h},{s}%,{l}%, 0.1)");

    let initials = fullname
        .split_whitespace()
        .filter_map(|word| word.get(0..1))
        .take(2)
        .collect::<String>()
        .to_uppercase();

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
            <circle cx="32" cy="32" r="32" fill=background_color />
            <text x="50%" y="50%" text-anchor="middle" dominant-baseline="central" font-size="24" fill=text_color font-weight="500">
                { initials }
            </text>
        </svg>
    }
}
