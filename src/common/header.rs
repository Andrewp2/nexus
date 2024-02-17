use leptos::{component, view, IntoView};
use leptos_router::A;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <img/>
            <nav class="nav">
                <A href="">"Home"</A>
                <A href="about">"About"</A>
                <A href="community">"Community"</A>
                <A href="support">"Help"</A>
                <A href="checkout" id="buy-game-nav">
                    "Buy Game"
                </A>
            </nav>
            <div class="authgroup">
                <A href="log_in">"Log in"</A>
                |
                <A href="log_in">"Sign up"</A>
            </div>
        </header>
    }
}

