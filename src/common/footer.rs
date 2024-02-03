use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class:footer=true>
            <div class="footeritems">
                Copyright 2023-2024 Andrew Peterson.
                <A href="terms_of_service">"Terms of Service"</A> <A href="credits">Credits</A>
                <A href="support">"Support / FAQ"</A>
            </div>
        </footer>
    }
}

