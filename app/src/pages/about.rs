use leptos::{component, view, IntoView};

#[component]
pub fn About() -> impl IntoView {
    view! {
        <h1 class="text-xl">"Who made this?"</h1>
        <p>"My name is Andrew Peterson. I'm just some guy."</p>
    }
}
