use leptos::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <h1>"$GAME_NAME is an exploration into the cave world of $WORLD_NAME."</h1>
        <p>Build, craft, explore, and conquer the ancient ruined cities beneath the earth.</p>
    }
}

