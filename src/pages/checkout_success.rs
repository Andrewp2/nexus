use leptos::{component, view, IntoView};

#[component]
pub fn CheckoutSuccess() -> impl IntoView {
    view! { <div>"Checkout was successful, you should be able to download the game now."</div> }
}

