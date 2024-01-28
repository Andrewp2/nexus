use leptos::*;

use crate::server::public::create_checkout;

#[component]
pub fn Checkout() -> impl IntoView {
    let checkout_resource = create_resource(|| (), |_| async move { create_checkout().await });
    view! {
        <h1>"Checkout Page"</h1>
        <script>
            r#"
            async function startStripeCheckout(clientSecret) {
               const stripe = Stripe('pk_test_51Nv3jfFM3dbbE2Esb8CT77AM5f6OWq9Q9NyshvfgS4ZlkWAiRQyhjM3MyqNKMfWSjStJoin5cbNVqulDyV2iTw48005un445Cs');
               await stripe.redirectToCheckout({sessionId: clientSecret});
            }
            "#
        </script>
        {move || match checkout_resource.get() {
            None => view! { <div>"Creating checkout page..."</div> },
            Some(client_secret) => {
                view! {
                    // let client_secret = id.clone();
                    <div>
                        <div id="checkout"></div>
                        <script>
                            {format!("startStripeCheckout('{}');", client_secret.unwrap())}
                        </script>
                    </div>
                }
            }
        }}
    }
}

