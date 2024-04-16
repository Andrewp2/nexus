use leptos::{component, create_resource, view, ErrorBoundary, IntoView, SignalGet, Suspense};

use crate::server::public::create_checkout;

#[component]
pub fn Checkout() -> impl IntoView {
    let checkout_resource = create_resource(|| (), |_| async move { create_checkout().await });

    let script = format!(
        "
    async function startStripeCheckout(clientSecret) {{
        const stripe = Stripe('{}');
        try {{
            let checkout = await stripe.initEmbeddedCheckout({{clientSecret: clientSecret}});
            checkout.mount('#checkout');
        }} catch (error) {{
            console.error(\"Checkout failed:\", error.message);
            alert(\"Checkout process failed. Please try again later.\");
        }}
    }}
    ",
        std::env!("STRIPE_PUBLIC_KEY")
    );
    view! {
        <h1>"Checkout"</h1>
        <script inner_html=script></script>
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || match checkout_resource.get() {
                None => view! { <div>"Creating checkout page..."</div> },
                Some(client_secret) => {
                    view! {
                        <div>
                            <div id="checkout"></div>
                            <ErrorBoundary fallback=|_errors| view! { <div class="error"></div> }>
                                <script>
                                    {format!(
                                        "startStripeCheckout('{}');",
                                        client_secret
                                            .expect("Able to get client secret from checkout creation"),
                                    )}

                                </script>
                            </ErrorBoundary>
                        </div>
                    }
                }
            }}

        </Suspense>
    }
}

