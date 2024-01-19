use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class:footer=true>
            <div class="footeritems">
                Copyright 2023-2024 Andrew Peterson.
                <A href="terms_and_conditions">Terms and conditions</A>
                <A href="credits">Credits</A> <A href="support">Support / FAQ</A>
            </div>
        </footer>
    }
}

