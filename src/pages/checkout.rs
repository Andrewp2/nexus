use leptos::*;

use crate::server::public::create_checkout;

#[component]
pub fn Checkout() -> impl IntoView {
    let checkout_resource =
        create_local_resource(|| (), |_| async move { create_checkout().await });

    let script = format!(
        "
    async function startStripeCheckout(clientSecret) {{
       const stripe = Stripe('{}');
       await stripe.redirectToCheckout({{sessionId: clientSecret}});
    }}
    ",
        std::env!("STRIPE_PUBLIC_KEY")
    );
    /*
        nexus.js:568 At src\pages\checkout.rs:21:42, you are reading a resource in `hydrate` mode outside a <Suspense/> or <Transition/>. This can cause hydration mismatch errors and loses out on a significant performance optimization. To fix this issue, you can either:
    1. Wrap the place where you read the resource in a <Suspense/> or <Transition/> component, or
    2. Switch to using create_local_resource(), which will wait to load the resource until the app is hydrated on the client side. (This will have worse performance in most cases.)
     */
    view! {
        <h1>"Checkout Page"</h1>
        <script inner_html=script></script>
        {move || match checkout_resource.get() {
            None => view! { <div>"Creating checkout page..."</div> },
            Some(client_secret) => {
                view! {
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

