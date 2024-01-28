use leptos::*;

use crate::server::public::CreateCheckout;

#[component]
pub fn Checkout() -> impl IntoView {
    let checkout_action = create_server_action::<CreateCheckout>();
    view! { <div id="checkout"></div> }
}

