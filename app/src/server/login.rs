use super::globals::{
    dynamo::constants::table_attributes::{
        EMAIL, EMAIL_VERIFIED, PASSWORD, SESSION_EXPIRY, SESSION_ID,
    },
    env_var::{get_host_prefix, get_table_name},
};
use super::utilities::{
    dynamo_client, handle_dynamo_generic_error, session_lifespan, verify_password,
};
use crate::errors::NexusError;
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::{
        query::{QueryError, QueryOutput},
        update_item::UpdateItemError,
    },
    types::AttributeValue,
};
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
    let columns_to_query = vec![EMAIL, PASSWORD, EMAIL_VERIFIED];
    let check_if_password_exists_filter_expression = format!("attribute_exists({})", PASSWORD);
    let key_condition = format!("{} = :email_val", EMAIL);
    let db_result = client
        .query()
        .limit(1)
        .table_name(get_table_name())
        .key_condition_expression(key_condition)
        .expression_attribute_values(":email_val", AttributeValue::S(email.clone()))
        .projection_expression(columns_to_query.join(", "))
        .filter_expression(check_if_password_exists_filter_expression)
        .send()
        .await
        .map_err(|e| aws_sdk_dynamodb::Error::from(e));

    let (password_database_hash, verified) = match db_result {
        Ok(val) => Ok(get_hash_and_verified_status_from_query(val)?),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }?;

    if !verified {
        log::error!("Not email verified");
        return Err(ServerFnError::from(NexusError::AccountNotVerified));
    }

    match verify_password(&password, &password_database_hash) {
        true => {
            let lifespan = session_lifespan(remember);
            let future_time = Utc::now() + lifespan;
            let session_uuid = Uuid::new_v4().to_string();
            let update_expression = format!(
                "SET {} = :session_id, {} = :session_expiry",
                SESSION_ID, SESSION_EXPIRY
            );
            let update_session_expiry_db_result = client
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
                .map_err(|e| aws_sdk_dynamodb::Error::from(e));

            match update_session_expiry_db_result {
                Ok(_) => {
                    let response = expect_context::<ResponseOptions>();
                    // Note that setting this cookie won't work in localhost (not HTTPS)
                    let cookie = format!(
                        "{}{}={};Expires={};Secure;SameSite=Strict;HttpOnly; Path=/",
                        get_host_prefix(),
                        SESSION_ID,
                        session_uuid.clone(),
                        future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
                    );
                    if let Ok(cookie) = HeaderValue::from_str(cookie.as_str()) {
                        response.append_header(header::SET_COOKIE, cookie);
                        return Ok(());
                    }
                    log::error!("Unable to create cookie {}", cookie);
                    Err(ServerFnError::from(NexusError::Unhandled))
                }
                Err(e) => Err(ServerFnError::from(match e {
                    aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_) => {
                        NexusError::EmailNotFoundLogin
                    }
                    _ => NexusError::GenericDynamoServiceError,
                })),
            }
        }
        // https://security.stackexchange.com/questions/227524/password-reset-giving-clues-of-possible-valid-email-addresses/227566#227566
        // TL;DR it is fine from a UX standpoint to say specifically they have the incorrect password, yes this does leak the fact
        // that a specific email address is logged in
        false => {
            log::error!("Tried to login with incorrect password");
            Err(ServerFnError::from(NexusError::IncorrectPassword))
        }
    }
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
