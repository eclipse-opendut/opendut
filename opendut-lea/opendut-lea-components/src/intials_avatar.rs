use leptos::prelude::*;
use crate::ProfilePictureColors;

pub enum ProfilePictureSize {
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
pub fn ProfilePicture(
    #[prop(into)] name: Signal<String>,
    size: ProfilePictureSize,
) -> impl IntoView {
    
    let initials = Signal::derive(move || {
        name.get()
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .flat_map(char::to_uppercase)
            .collect::<String>()
    });
    
    let hsl = Signal::derive(move || {
        let colors = ProfilePictureColors::get_vec();
        let sum: u64 = name.get().chars().map(|c| c as u64).sum();
        let index = (sum % colors.len() as u64) as usize;
        colors[index].get_hsl()
    });

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
