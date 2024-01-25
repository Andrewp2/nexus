use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="headerlinks">
                <img/>
                <nav class="nav">
                    <A href="">"Home"</A>
                    <A href="about">"About"</A>
                    <A href="community">"Community"</A>
                    <A href="support">"Help"</A>
                </nav>
                <div class="authgroup">
                    <A href="log_in">"Log in"</A>
                    |
                    <A href="log_in">"Sign up"</A>
                </div>
            </div>
        </header>
    }
}

