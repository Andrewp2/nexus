use leptos::{component, view, IntoView};
use leptos_router::A;

#[component]
pub fn Community() -> impl IntoView {
    view! {
        <A class="discord-link" href="https://discord.gg/EuqSvxDPRY">
            "Join the official Discord!"
        </A>
    }
}
