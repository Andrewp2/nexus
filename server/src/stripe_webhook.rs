use aws_sdk_dynamodb::types::AttributeValue;
use axum::{
    body::{Body, HttpBody},
    extract::FromRequest,
    http::Request,
    response::{IntoResponse, Response},
    Extension,
};
use headers::Header;
use http::{HeaderName, HeaderValue, StatusCode};
use std::{env, fmt::Debug};
use stripe::{CheckoutSession, Event as WebhookEvent, EventObject, EventType, Webhook};

use app::server::globals::{
    app_state::AppState,
    dynamo::update_setup,
};

impl From<(StatusCode, String)> for ServerError {
    fn from(value: (StatusCode, String)) -> Self {
        Self(value.0, value.1)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

pub fn get_stripe_webhook_signature() -> String {
    match env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot get STRIPE_WEBHOOK_SECRET");
        }
    }
}

#[derive(Debug)]
pub struct ServerError(pub StatusCode, pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StripeSignatureHeader(pub String);

static STRIPE_SIGNATURE_HEADER: HeaderName = HeaderName::from_static("stripe-signature");
impl Header for StripeSignatureHeader {
    fn name() -> &'static HeaderName {
        &STRIPE_SIGNATURE_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        Ok(Self(
            value
                .to_str()
                .map_err(|_| headers::Error::invalid())?
                .to_string(),
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        if let Ok(val) = HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(val));
        }
    }
}

const MAX_ALLOWED_REQ_SIZE: u64 = 1_000_000;
pub struct SignedStripeEvent(pub WebhookEvent);

pub fn handle_error(err: impl Debug) -> (StatusCode, String) {
    tracing::error!("{:?}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".into(),
    )
}

#[async_trait::async_trait]
impl<S: Sync> FromRequest<S, Body> for SignedStripeEvent {
    type Rejection = ServerError;

    async fn from_request(req: Request<Body>, _: &S) -> Result<Self, Self::Rejection> {
        let signature = req
            .headers()
            .get("Stripe-Signature")
            .ok_or_else(|| {
                ServerError(
                    StatusCode::BAD_REQUEST,
                    "Missing Stripe-Signature header".into(),
                )
            })
            .and_then(|value| {
                value.to_str().map_err(|_| {
                    ServerError(
                        StatusCode::BAD_REQUEST,
                        "Invalid Stripe-Signature header".into(),
                    )
                })
            })
            .map_err(handle_error)?
            .to_string();
        let secret = get_stripe_webhook_signature();
        let req_content_length = match req.body().size_hint().upper() {
            Some(v) => v,
            None => MAX_ALLOWED_REQ_SIZE + 1, // Just to protect ourselves from a malicious response
        };
        if req_content_length < MAX_ALLOWED_REQ_SIZE {
            let body = axum::body::to_bytes(req.into_body(), req_content_length as usize)
                .await
                .map_err(handle_error)?;
            let body_str = std::str::from_utf8(&body).map_err(handle_error)?;

            Ok(SignedStripeEvent(
                Webhook::construct_event(body_str, &signature, &secret)
                    .map_err(|err| ServerError(StatusCode::UNAUTHORIZED, format!("{:?}", err)))?,
            ))
        } else {
            Err(ServerError(StatusCode::PAYLOAD_TOO_LARGE, "...".into()))
        }
    }
}

pub fn not_found<S: AsRef<str>>(item: S) -> (StatusCode, String) {
    (
        StatusCode::NOT_FOUND,
        format!("{} not found.", item.as_ref()),
    )
}

async fn checkout_session_completed(
    dynamo_client: &aws_sdk_dynamodb::Client,
    checkout_session: CheckoutSession,
) -> Result<(), (StatusCode, String)> {
    let metadata = checkout_session.metadata.unwrap();
    let email = checkout_session
        .customer_email
        .ok_or(not_found("customer_email"))?;
    let item_id = metadata
        .get("item_id")
        .ok_or(not_found("item_id metadata"))?;
    let update = update_setup(dynamo_client, email)
        .update_expression("SET #listAttr = list_append(#listAttr, :newElement)")
        .expression_attribute_names("#listAttr", "listAttributeName")
        .expression_attribute_values(
            ":newElement",
            AttributeValue::L(vec![AttributeValue::S(item_id.to_string())]),
        )
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);
    match update {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Could not update dynamo table after checkout session!!!! This is really important!!! {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not update database".to_string(),
            ))
        }
    }
}

async fn process_checkout(
    dynamo_client: &aws_sdk_dynamodb::Client,
    _stripe_client: &stripe::Client,
    checkout_session: CheckoutSession,
    event_type: EventType,
) -> Result<(), (StatusCode, String)> {
    match event_type {
        EventType::CheckoutSessionAsyncPaymentFailed => Ok(()),
        EventType::CheckoutSessionAsyncPaymentSucceeded => Ok(()),
        EventType::CheckoutSessionCompleted => {
            checkout_session_completed(dynamo_client, checkout_session).await
        }
        EventType::CheckoutSessionExpired => Ok(()),
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Wrong event type for event associated with checkout".into(),
        )),
    }
}

pub async fn stripe_webhook(
    Extension(state): Extension<AppState>,
    SignedStripeEvent(event): SignedStripeEvent,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let stripe_client = state.stripe_client;
    let dynamo_client = state.dynamodb_client;
    let event_type = event.type_;
    let _idempotency_key = event.request.and_then(|req| req.idempotency_key);
    match event.data.object {
        EventObject::CheckoutSession(checkout) => {
            process_checkout(&dynamo_client, &stripe_client, checkout, event_type).await?;
        }
        //TODO: HANDLE DISPUTE
        EventObject::Dispute(dispute) => {
            log::error!("There was a dispute!!!");
            log::error!("{:?}", dispute);
            //process_dispute(&db, stripe, dispute, event_type).await?;
        }
        EventObject::Subscription(_) => {}
        EventObject::Charge(_) => {}
        EventObject::Account(_) => {}
        EventObject::AccountCapabilities(_) => {}
        EventObject::Application(_) => {}
        EventObject::ApplicationFee(_) => {}
        EventObject::ApplicationFeeRefund(_) => {}
        EventObject::Balance(_) => {}
        EventObject::BankAccount(_) => {}
        EventObject::BillingPortalConfiguration(_) => {}
        EventObject::Card(_) => {}
        EventObject::Coupon(_) => {}
        EventObject::Customer(_) => {}
        EventObject::Discount(_) => {}
        EventObject::File(_) => {}
        EventObject::Invoice(_) => {}
        EventObject::InvoiceItem(_) => {}
        EventObject::IssuingAuthorization(_) => {}
        EventObject::IssuingCard(_) => {}
        EventObject::IssuingCardholder(_) => {}
        EventObject::IssuingDispute(_) => {}
        EventObject::IssuingTransaction(_) => {}
        EventObject::Mandate(_) => {}
        EventObject::PaymentIntent(_) => {}
        EventObject::PaymentLink(_) => {}
        EventObject::PaymentMethod(_) => {}
        EventObject::Payout(_) => {}
        EventObject::Person(_) => {}
        EventObject::Plan(_) => {}
        EventObject::Price(_) => {}
        EventObject::Product(_) => {}
        EventObject::PromotionCode(_) => {}
        EventObject::Quote(_) => {}
        EventObject::Refund(_) => {}
        EventObject::Review(_) => {}
        EventObject::SetupIntent(_) => {}
        EventObject::SubscriptionSchedule(_) => {}
        EventObject::TaxId(_) => {}
        EventObject::TaxRate(_) => {}
        EventObject::TestHelpersTestClock(_) => {}
        EventObject::Topup(_) => {}
        EventObject::Transfer(_) => {}
    }
    Ok(())
}
