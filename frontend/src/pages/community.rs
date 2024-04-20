use leptos::{component, view, IntoView};
use leptos_router::A;

#[component]
pub fn Community() -> impl IntoView {
    view! {
        <A
            class="transition-colors duration-300 ease-in-out text-white py-1.5 px-1.5 bg-[#primary-color] rounded-md hover:bg-hover-background-color"
            href="https://discord.gg/EuqSvxDPRY"
        >
            "Join the official Discord!"
        </A>
    }
}

