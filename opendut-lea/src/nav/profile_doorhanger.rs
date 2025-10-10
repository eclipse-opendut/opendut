use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, Doorhanger, DoorhangerAlignment, FontAwesomeIcon, IconButton};
use tracing::log;
use crate::user::UserAuthenticationSignal;

#[component]
pub fn ProfileDoorhanger() -> impl IntoView {

    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");

    let fullname = user.read().fullname();
    let username = user.read().username();
    let email = user.read().email();

    let fullname = Some(String::from("Salvatore Fendt"));
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
                        <span class="is-flex">
                            <p class="dut-fullname-text pr-2"> { fullname }</p>
                            <p class="dut-username-text"> { username } </p>
                        </span>
                        <p class="dut-email-text"> { email } </p>
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

    let hsl_colors = vec![
        "355, 76, 36",  // dark-red
        "356, 83, 41",  // red
        "31, 100, 48",  // orange
        "41, 100, 53",  // yellow
        "78, 51, 56",   // light-green
        "146, 43, 30",  // dark-green
        "173, 57, 73",  // light-blue
        "217, 100, 56", // blue
        "220, 54, 25",  // dark-blue
        "294, 24, 53",  // purple
        "334, 100, 50", // pink
        "228, 21, 72",  // grey
    ];

    let sum: u32 = fullname.chars().map(|c| c as u32).sum();
    let index = (sum % hsl_colors.len() as u32) as usize;
    let color = hsl_colors[index];
    log::info!("color: {color}");
    log::info!("fullname: {fullname}");

    let initials = fullname
        .split_whitespace()
        .filter_map(|word| word.get(0..1)).take(2)
        .collect::<String>()
        .to_uppercase();

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
            <circle cx="32" cy="32" r="32" fill=format!("hsla({color}, 0.1)") />
            <text x="50%" y="50%" text-anchor="middle" dominant-baseline="central" font-size="24" fill=format!("hsl({color})")>
                { initials }
            </text>
        </svg>
    }
}
