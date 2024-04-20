use leptos::{component, view, IntoView};

#[component]
pub fn EmailVerification() -> impl IntoView {
    view! {
        <h1>
            "You should be recieving an email to the email address you specified when logging in."
        </h1>
        <h2>"Click on that link, and you can log in as you wish."</h2>
    }
}

