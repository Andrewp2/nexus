use super::csrf::{generate_csrf_token, generate_random_bytes};
use super::globals::dynamo::{query_builder, query_setup, TableKeyType};
use super::utilities::{
    dynamo_client, handle_dynamo_generic_error, session_lifespan, verify_password,
};
use super::{
    globals::{
        dynamo::constants::table_attributes::{
            EMAIL, EMAIL_VERIFIED, PASSWORD, SESSION_EXPIRY, SESSION_ID,
        },
        env_var::{get_host_prefix, get_table_name},
    },
    utilities::kms_client,
};
use crate::errors::NexusError;
use aws_sdk_dynamodb::{operation::query::QueryOutput, types::AttributeValue};
use aws_sdk_kms::Client as KeyClient;
use chrono::Utc;
use http::{header, HeaderValue};
use leptos::{expect_context, ServerFnError};
use leptos_axum::ResponseOptions;
use uuid::Uuid;

pub async fn login(
    email: String,
    password: String,
    remember: bool,
) -> Result<(), ServerFnError<NexusError>> {
    let client = dynamo_client()?;
    let kms_client = kms_client()?;
    let columns_to_query = [EMAIL, PASSWORD, EMAIL_VERIFIED];
    let check_if_password_exists_filter_expression = format!("attribute_exists({})", PASSWORD);
    let key_condition = format!("{} = :email_val", EMAIL);
    let db_result = query_setup(&client, email.clone(), TableKeyType::Email)
        .key_condition_expression(key_condition)
        .projection_expression(columns_to_query.join(", "))
        .filter_expression(check_if_password_exists_filter_expression)
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);
    let (password_database_hash, verified) = match db_result {
        Ok(val) => Ok(get_hash_and_verified_status_from_query(val)?),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }?;
    if !verified {
        log::error!("Not email verified");
        return Err(ServerFnError::from(NexusError::AccountNotVerified));
    }
    match verify_password(&password, &password_database_hash) {
        true => update_session_and_set_cookie(remember, kms_client, client, email).await,
        // https://security.stackexchange.com/questions/227524/password-reset-giving-clues-of-possible-valid-email-addresses/227566#227566
        // TL;DR it is fine from a UX standpoint to say specifically they have the incorrect password, yes this does leak the fact
        // that a specific email address is logged in
        false => {
            log::error!("Tried to login with incorrect password");
            Err(ServerFnError::from(NexusError::IncorrectPassword))
        }
    }
}

async fn update_session_and_set_cookie(
    remember: bool,
    kms_client: std::sync::Arc<KeyClient>,
    dynamo_client: std::sync::Arc<aws_sdk_dynamodb::Client>,
    email: String,
) -> Result<(), ServerFnError<NexusError>> {
    let lifespan = session_lifespan(remember);
    let future_time = Utc::now() + lifespan;
    let session_uuid = Uuid::new_v4().to_string();
    let update_expression = format!(
        "SET {} = :session_id, {} = :session_expiry",
        SESSION_ID, SESSION_EXPIRY
    );
    let kms_client: &aws_sdk_kms::Client = &kms_client;
    let random_bytes = generate_random_bytes();
    let csrf_token = generate_csrf_token(kms_client, session_uuid.clone(), random_bytes).await?;
    let update_session_expiry_db_result = dynamo_client
        .update_item()
        .table_name(get_table_name())
        .key(EMAIL, AttributeValue::S(email))
        .update_expression(update_expression)
        .expression_attribute_values(":session_id", AttributeValue::S(session_uuid.clone()))
        .expression_attribute_values(
            ":session_expiry",
            AttributeValue::N(future_time.timestamp().to_string()),
        )
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);

    match update_session_expiry_db_result {
        Ok(_) => on_successful_session_update(session_uuid, future_time, csrf_token),
        Err(e) => Err(ServerFnError::from(match e {
            aws_sdk_dynamodb::Error::ConditionalCheckFailedException(e) => {
                #[cfg(debug_assertions)]
                log::error!("ConditionalCheckFailedException (email not found?) {:?}", e);
                NexusError::EmailNotFoundLogin
            }
            e => {
                #[cfg(debug_assertions)]
                log::error!("Generic error {:?}", e);
                NexusError::GenericDynamoServiceError
            }
        })),
    }
}

fn on_successful_session_update(
    session_uuid: String,
    future_time: chrono::DateTime<Utc>,
    csrf_token: String,
) -> Result<(), ServerFnError<NexusError>> {
    let response = expect_context::<ResponseOptions>();
    // Note that setting this cookie won't work in localhost (not HTTPS)
    let session_cookie = format!(
        "{}{}={};Expires={};Secure;SameSite=Lax;HttpOnly; Path=/",
        get_host_prefix(),
        SESSION_ID,
        session_uuid.clone(),
        future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    );
    // no HTTPOnly flag
    let csrf_cookie = format!(
        "{}{}={};Expires={};Secure;SameSite=Lax; Path=/",
        get_host_prefix(),
        "csrf_token",
        csrf_token,
        future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    );
    if let Ok(session_cookie) = HeaderValue::from_str(session_cookie.as_str()) {
        if let Ok(csrf_cookie) = HeaderValue::from_str(csrf_cookie.as_str()) {
            response.append_header(header::SET_COOKIE, session_cookie);
            response.append_header(header::SET_COOKIE, csrf_cookie);
            leptos_axum::redirect("/");
        }
    }
    log::error!(
        "Unable to create cookie {} or {}",
        session_cookie,
        csrf_cookie
    );
    Err(ServerFnError::from(NexusError::Unhandled))
}

fn get_hash_and_verified_status_from_query(
    val: QueryOutput,
) -> Result<(String, bool), ServerFnError<NexusError>> {
    let item = val.items().first().ok_or(ServerFnError::from(
        NexusError::CouldNotFindRowWithThatEmail,
    ))?;
    let hash_string = item
        .get(PASSWORD)
        .ok_or_else(|| {
            log::error!("Was not able to find the password, despite the filter expression");
            ServerFnError::from(NexusError::GenericDynamoServiceError)
        })?
        .as_s()
        .map_err(|e| {
            log::error!(
                "Was not able to get the inner blob from the password {:?}",
                e
            );
            ServerFnError::from(NexusError::Unhandled)
        })?
        .to_owned();
    let email_verified = item
        .get(EMAIL_VERIFIED)
        .ok_or_else(|| {
            log::error!("Was not able to get whether or not this email verification status");
            ServerFnError::from(NexusError::Unhandled)
        })?
        .as_bool()
        .map_err(|e| {
            log::error!("Could not convert EMAIL_VERIFIED to boolean {:?}", e);
            ServerFnError::from(NexusError::Unhandled)
        })?;
    Ok((hash_string, *email_verified))
}
