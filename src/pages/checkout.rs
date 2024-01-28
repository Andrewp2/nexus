use leptos::*;

use crate::server::public::create_checkout;

#[component]
pub fn Checkout() -> impl IntoView {
    let checkout_resource = create_resource(|| (), |_| async move { create_checkout().await });

    let script = format!(
        "
    async function startStripeCheckout(clientSecret) {{
       const stripe = Stripe('{}');
       await stripe.redirectToCheckout({{sessionId: clientSecret}});
    }}
    ",
        std::env!("STRIPE_PUBLIC_KEY")
    );
    view! {
        <h1>"Checkout Page"</h1>
        <script inner_html=script></script>
        {move || match checkout_resource.get() {
            None => view! { <div>"Creating checkout page..."</div> },
            Some(client_secret) => {
                view! {
                    // let client_secret = id.clone();
                    <div>
                        <div id="checkout"></div>
                        <ErrorBoundary fallback=|errors| view! { <div class="error"></div> }>
                            <script>
                                {format!("startStripeCheckout('{}');", client_secret.unwrap())}
                            </script>
                        </ErrorBoundary>
                    </div>
                }
            }
        }}
    }
}

