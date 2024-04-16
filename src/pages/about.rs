use leptos::{component, view, IntoView};

#[component]
pub fn About() -> impl IntoView {
    view! {
        <h1 class="h1">"$GAME_NAME is an exploration into the cave world of $WORLD_NAME."</h1>
        <p>Build, craft, explore, and conquer the ancient ruined cities beneath the earth.</p>
        <h2 class="h2">"Who made this?"</h2>
        <p>
            "My name is Andrew Peterson.
             I've been working on gamedev for about the past 2 years, and this is the website where I host the game I'm making."
        </p>
    }
}

