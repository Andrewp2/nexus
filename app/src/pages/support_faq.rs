use leptos::{component, view, IntoView};

#[component]
pub fn SupportFAQ() -> impl IntoView {
    let list_items: Vec<&str> = vec![
        "The game works best on Firefox, next best on Chrome, and last on Safari. If you run into an issue, try running it on a different browser.",
        "If you have any issues with authentication, please contact my email directly.",
        "If you have any GDPR request or want to delete your account, please contact my email. \
        I'm working on automating this so this is unnecessary."
    ];
    view! {
        <h1 class="text-2xl font-bold">Frequently Asked Questions</h1>
        <br/>
        <ul class="list-disc pl-8">
            {list_items
                .into_iter()
                .map(|n| {
                    view! {
                        <li>{n}</li>
                        <br/>
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
        <p>"Contact andrew@ProjectGlint.com for any issues with support not handled here."</p>
    }
}
