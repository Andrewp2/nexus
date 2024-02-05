use super::utilities::{dynamo_client, session_lifespan, verify_password};
use crate::{
    dynamo::constants::table_attributes::{
        EMAIL, EMAIL_VERIFIED, PASSWORD, SESSION_EXPIRY, SESSION_ID,
    },
    env_var::get_table_name,
    errors::NexusError,
};
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
        .await;

    let (password_database_hash, verified) = match db_result {
        Ok(val) => Ok(get_hash_and_verified_status_from_query(val)?),
        Err(e) => Err({
            let ne = aws_sdk_dynamodb::Error::from(e);
            log::error!("{:?}", ne);
            ServerFnError::from(NexusError::GenericDynamoServiceError)
        }),
    }?;

    if !verified {
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
            let db_result = client
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
                .await;
            match db_result {
                Ok(_) => {
                    let response = expect_context::<ResponseOptions>();
                    // INFO: time expiration format seems correct since the original time is in UTC,
                    // and GMT == UTC for HTTP purposes
                    // double check this in the future
                    let cookie = format!(
                        "__Host-{}={};Expires={};Secure;SameSite=Strict;HttpOnly; Path=/",
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
                Err(e) => Err(handle_login_update_error(e)),
            }
        }
        // https://security.stackexchange.com/questions/227524/password-reset-giving-clues-of-possible-valid-email-addresses/227566#227566
        // TL;DR it is fine from a UX standpoint to say specifically they have the incorrect password, yes this does leak the fact
        // that a specific email address is logged in
        false => Err(ServerFnError::from(NexusError::IncorrectPassword)),
    }
}

fn get_hash_and_verified_status_from_query(
    val: QueryOutput,
) -> Result<(String, bool), ServerFnError<NexusError>> {
    let item = val.items().first().ok_or(ServerFnError::from(
        NexusError::CouldNotFindRowWithThatEmail,
    ))?;
    let blob = item
        .get(PASSWORD)
        .ok_or_else(|| -> ServerFnError {
            log::error!("Was not able to find the password, despite the filter expression");
            ServerFnError::new(NexusError::GenericDynamoServiceError)
        })?
        .as_b()
        .map_err(|e| -> ServerFnError {
            log::error!(
                "Was not able to get the inner blob from the password {:?}",
                e
            );
            ServerFnError::new(NexusError::Unhandled)
        })?;
    let hash_string =
        String::from_utf8(blob.clone().into_inner()).map_err(|e| -> ServerFnError {
            log::error!(
                "Was not able to get a utf8 string from the blob {:?}, {:?}",
                blob,
                e
            );
            ServerFnError::new(NexusError::Unhandled)
        })?;
    let email_verified = item
        .get(EMAIL_VERIFIED)
        .ok_or_else(|| -> ServerFnError {
            log::error!("Was not able to get whether or not this email verification status");
            ServerFnError::new(NexusError::Unhandled)
        })?
        .as_bool()
        .map_err(|e| {
            log::error!("Could not convert EMAIL_VERIFIED to boolean {:?}", e);
            ServerFnError::from(NexusError::Unhandled)
        })?;
    Ok((hash_string, *email_verified))
}

fn handle_login_update_error(e: SdkError<UpdateItemError>) -> ServerFnError<NexusError> {
    ServerFnError::from(match e.into_service_error() {
        UpdateItemError::ConditionalCheckFailedException(e) => {
            // TODO: Check if this needs to be logged, as this occurs when we could not update the session_id/session_expiry
            log::error!("{:?}", e);
            NexusError::EmailNotFoundLogin
        }
        UpdateItemError::InternalServerError(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        UpdateItemError::InvalidEndpointException(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        UpdateItemError::ItemCollectionSizeLimitExceededException(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        UpdateItemError::ProvisionedThroughputExceededException(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        UpdateItemError::RequestLimitExceeded(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        UpdateItemError::ResourceNotFoundException(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        UpdateItemError::TransactionConflictException(e) => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
        e => {
            log::error!("{:?}", e);
            NexusError::GenericDynamoServiceError
        }
    })
}

