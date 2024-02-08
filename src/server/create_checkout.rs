use leptos::ServerFnError;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCustomer, CreatePrice, CreateProduct, Currency, Customer,
    Expandable, IdOrCreate, Price, Product,
};

use crate::{
    errors::NexusError,
    server::utilities::{
        check_if_session_is_valid, dynamo_client, get_session_cookie, stripe_client,
    },
    site::constants::SITE_FULL_DOMAIN,
};

pub async fn create_checkout() -> Result<String, ServerFnError<NexusError>> {
    log::error!("hello");
    let stripe_client = stripe_client()?;
    #[cfg(debug_assertions)]
    let email = "example@example.com".to_owned();
    #[cfg(not(debug_assertions))]
    {
        log::error!("release");
        let session_id_cookie = get_session_cookie().await?;
        let dynamo_client = dynamo_client()?;
        let (valid, email) = check_if_session_is_valid(session_id_cookie, &dynamo_client).await?;
        if !valid {
            return Err(ServerFnError::from(NexusError::Unhandled));
        }
    }
    log::error!("hello 2");
    let customer = Customer::create(
        &stripe_client,
        CreateCustomer {
            email: Some(email.as_str()),
            metadata: Some(std::collections::HashMap::from([(
                String::from("async-stripe"),
                String::from("true"),
            )])),

            ..Default::default()
        },
    )
    .await
    .map_err(|e| {
        log::error!("{:?}", e);
        ServerFnError::from(NexusError::Unhandled)
    })?;

    log::error!("hello 3");

    log::info!(
        "created a customer at https://dashboard.stripe.com/test/customers/{}",
        customer.id
    );
    // finally, create a checkout session for this product / price
    let mut params = CreateCheckoutSession::new();
    let redirect_url = format!("https://{}/download", SITE_FULL_DOMAIN);
    params.return_url = Some(&redirect_url);
    params.customer = Some(customer.id);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price: Some("price_1OgmNmFM3dbbE2EswStQyvJa".to_string()),
        ..Default::default()
    }]);
    params.expand = &["line_items", "line_items.data.price.product"];
    params.ui_mode = Some(stripe::CheckoutSessionUiMode::Embedded);
    log::error!("hello");
    let checkout_session = CheckoutSession::create(&stripe_client, params)
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            ServerFnError::from(NexusError::Unhandled)
        })?;

    log::error!("hello 4");
    return match checkout_session.client_secret {
        Some(secret) => Ok(secret),
        None => Err(ServerFnError::from(NexusError::Unhandled)),
    };
}

