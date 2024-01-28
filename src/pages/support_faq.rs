use leptos::*;

#[component]
pub fn SupportFAQ() -> impl IntoView {
    view! {
        <h1>FAQ</h1>
        <ul>
            <li>
                "The game works best on Firefox, next best on Chrome, and worst on Safari. If you run into an issue, try running it on a different browser."
            </li>
            <li>"If you have any issue with authentication, please contact my email directly."</li>
            <li>
                "If you have any GDPR request, or request to delete your account, please contact my email directly. I'm working on automating this so this is unnecessary."
            </li>
        </ul>
        <p>Contact andrew@MySite.com for any issues with support not handled here.</p>
    }
}

