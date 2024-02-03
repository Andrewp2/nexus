use leptos::ServerFnError;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCustomer, CreatePrice, CreateProduct, Currency, Customer,
    Expandable, IdOrCreate, Price, Product,
};

use crate::{
    dynamo::constants::get_table_name,
    errors::NexusError,
    server::utilities::{
        check_if_session_is_valid, dynamo_client, get_session_cookie, stripe_client,
    },
    site::constants::SITE_FULL_DOMAIN,
};

const PRICE_OF_GAME_IN_CENTS: i64 = 3000;

pub async fn create_checkout() -> Result<String, ServerFnError> {
    // TODO: uncomment
    // let session_id_cookie = get_session_cookie().await?;
    // let dynamo_client = dynamo_client()?;
    // let (valid, email) = check_if_session_is_valid(session_id_cookie, &dynamo_client).await?;
    // if !valid {
    //     return Err(ServerFnError::new(
    //         NexusError::Unhandled,
    //     ));
    // }
    let stripe_client = stripe_client()?;

    let customer = Customer::create(
        &stripe_client,
        CreateCustomer {
            //email: Some(email.as_str()),
            email: Some("example@example.com"),
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
    .map_err(|e| {
        log::error!("{:?}", e);
        ServerFnError::new(NexusError::Unhandled)
    })?;

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
    let product = Product::create(&stripe_client, create_product)
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            ServerFnError::new(NexusError::Unhandled)
        })?;

    // and add a price for it in USD
    let mut create_price = CreatePrice::new(Currency::USD);
    create_price.product = Some(IdOrCreate::Id(&product.id));
    create_price.metadata = Some(std::collections::HashMap::from([(
        String::from("async-stripe"),
        String::from("true"),
    )]));
    create_price.unit_amount = Some(PRICE_OF_GAME_IN_CENTS);
    create_price.expand = &["product"];
    let price = Price::create(&stripe_client, create_price)
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            ServerFnError::new(NexusError::Unhandled)
        })?;

    log::info!(
        "created a product {:?} at price {} {}",
        product.name.unwrap_or("Product has no name!".to_owned()),
        price.unit_amount.unwrap_or(PRICE_OF_GAME_IN_CENTS) / 100,
        price.currency.unwrap_or(Currency::USD)
    );

    // finally, create a checkout session for this product / price
    let mut params = CreateCheckoutSession::new();
    //let cancel_url = format!("http://{}/cancel", SITE_FULL_DOMAIN);
    //params.cancel_url = Some(&cancel_url);
    let redirect_url = format!("http://{}/download", SITE_FULL_DOMAIN);
    params.return_url = Some(&redirect_url);
    params.customer = Some(customer.id);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price: Some(price.id.to_string()),
        ..Default::default()
    }]);
    params.expand = &["line_items", "line_items.data.price.product"];
    params.ui_mode = Some(stripe::CheckoutSessionUiMode::Embedded);

    let checkout_session = CheckoutSession::create(&stripe_client, params)
        .await
        .map_err(|e| {
            log::error!("{:?}", e);
            ServerFnError::new(NexusError::Unhandled)
        })?;
    log::info!("return");
    return match checkout_session.client_secret {
        Some(secret) => Ok(secret),
        None => Err(ServerFnError::new(NexusError::Unhandled)),
    };
}

