use leptos::ServerFnError;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCustomer, CreatePrice, CreateProduct, Currency, Customer,
    Expandable, IdOrCreate, Price, Product,
};

use crate::{
    dynamo::constants::get_table_name,
    errors::NexusError,
    server::utilities::{check_if_session_is_valid, dynamo_client, get_session_cookie},
    site::constants::SITE_FULL_DOMAIN,
};

pub async fn create_checkout() -> Result<String, ServerFnError> {
    let session_id_cookie = get_session_cookie().await?;
    let dynamo_client = dynamo_client()?;
    let (valid, email) = check_if_session_is_valid(session_id_cookie, &dynamo_client).await?;
    if !valid {
        return Err(ServerFnError::ServerError(
            NexusError::Unhandled.to_string(),
        ));
    }
    let secret_key = std::env::var("STRIPE_SECRET_KEY").expect("Missing STRIPE_SECRET_KEY in env");
    let client = Client::new(secret_key);

    let customer = Customer::create(
        &client,
        CreateCustomer {
            email: Some(email.as_str()),
            description: Some(
                "A fake customer that is used to illustrate the examples in async-stripe.",
            ),
            metadata: Some(std::collections::HashMap::from([(
                String::from("async-stripe"),
                String::from("true"),
            )])),

            ..Default::default()
        },
    )
    .await
    .map_err(|_| ServerFnError::ServerError(NexusError::Unhandled.to_string()))?;

    log::info!(
        "created a customer at https://dashboard.stripe.com/test/customers/{}",
        customer.id
    );

    // create a new example project
    let mut create_product = CreateProduct::new("video game");
    create_product.metadata = Some(std::collections::HashMap::from([(
        String::from("async-stripe"),
        String::from("true"),
    )]));
    let product = Product::create(&client, create_product)
        .await
        .map_err(|_| ServerFnError::ServerError(NexusError::Unhandled.to_string()))?;

    // and add a price for it in USD
    let mut create_price = CreatePrice::new(Currency::USD);
    create_price.product = Some(IdOrCreate::Id(&product.id));
    create_price.metadata = Some(std::collections::HashMap::from([(
        String::from("async-stripe"),
        String::from("true"),
    )]));
    create_price.unit_amount = Some(3000);
    create_price.expand = &["product"];
    let price = Price::create(&client, create_price)
        .await
        .map_err(|_| ServerFnError::ServerError(NexusError::Unhandled.to_string()))?;

    log::info!(
        "created a product {:?} at price {} {}",
        product.name.unwrap_or("Product has no name!".to_owned()),
        price.unit_amount.unwrap_or(0i64) / 100,
        price.currency.unwrap_or(Currency::USD)
    );

    // finally, create a checkout session for this product / price
    let mut params = CreateCheckoutSession::new();
    let cancel_url = format!("http://{}/cancel", SITE_FULL_DOMAIN);
    params.cancel_url = Some(&cancel_url);
    params.customer = Some(customer.id);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price: Some(price.id.to_string()),
        ..Default::default()
    }]);
    params.expand = &["line_items", "line_items.data.price.product"];
    params.ui_mode = Some(stripe::CheckoutSessionUiMode::Embedded);

    let checkout_session = CheckoutSession::create(&client, params)
        .await
        .map_err(|_| ServerFnError::ServerError(NexusError::Unhandled.to_string()))?;

    return match checkout_session.client_secret {
        Some(secret) => Ok(secret),
        None => Err(ServerFnError::ServerError(
            NexusError::Unhandled.to_string(),
        )),
    };
}

