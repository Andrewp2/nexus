use std::{collections::HashMap, sync::Arc};

use argon2::{
    password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use aws_sdk_dynamodb::{
    operation::query::QueryOutput, types::AttributeValue, Client as DynamoClient,
};
use aws_sdk_kms::Client as KeyClient;
use aws_sdk_ses::Client as SesClient;
use axum_extra::extract::CookieJar;
use chrono::Utc;
use leptos::{use_context, ServerFnError};
use leptos_axum::extract;
use rand::rngs::OsRng;
use stripe::Client as StripeClient;

use super::globals::{
    self,
    dynamo::{
        constants::table_attributes::{self, EMAIL, SESSION_EXPIRY, SESSION_ID},
        query_builder, query_setup, TableKeyType,
    },
    env_var::{get_host_prefix, get_table_name},
};

use crate::errors::{NexusError, UNHANDLED};

pub fn dynamo_client() -> Result<Arc<DynamoClient>, ServerFnError<NexusError>> {
    use_context::<Arc<DynamoClient>>().ok_or_else(|| UNHANDLED)
}

pub fn ses_client() -> Result<Arc<SesClient>, ServerFnError<NexusError>> {
    use_context::<Arc<SesClient>>().ok_or_else(|| UNHANDLED)
}

pub fn kms_client() -> Result<Arc<KeyClient>, ServerFnError<NexusError>> {
    use_context::<Arc<KeyClient>>().ok_or_else(|| UNHANDLED)
}

pub fn stripe_client() -> Result<Arc<StripeClient>, ServerFnError<NexusError>> {
    use_context::<Arc<StripeClient>>().ok_or_else(|| {
        log::error!("Could not get Stripe client");
        UNHANDLED
    })
}

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let string = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(string)
}

pub fn verify_password(password: &str, database_hash: &str) -> bool {
    if let Ok(hash) = PasswordHash::new(database_hash) {
        return Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok();
    }
    false
}

pub fn session_lifespan(remember: bool) -> chrono::Duration {
    match remember {
        true => chrono::Duration::hours(3),
        false => chrono::Duration::days(60),
    }
}

pub async fn get_session_cookie() -> Result<String, ServerFnError<NexusError>> {
    let cookie_jar: CookieJar = extract().await.map_err(|e| {
        log::error!("Could not get cookie jar {:?}", e);
        NexusError::Unhandled
    })?;
    let session_id = cookie_jar
        .get(format!("{}{}", get_host_prefix(), SESSION_ID).as_str())
        .ok_or_else(|| {
            log::error!("Couldn't get session_id from cookie_jar");
            UNHANDLED
        })?;
    let f = session_id
        .value()
        .to_string()
        .strip_prefix(get_host_prefix())
        .ok_or_else(|| {
            log::error!("Couldn't remove __Host- prefix from cookie");
            UNHANDLED
        })?
        .to_string();

    Ok(f)
}

pub async fn check_if_session_is_valid(
    session_id_cookie: String,
    csrf_cookie: String,
    dynamo_client: &DynamoClient,
    kms_client: &KeyClient,
) -> Result<(bool, String), ServerFnError<NexusError>> {
    let query = query_setup(
        dynamo_client,
        session_id_cookie.clone(),
        TableKeyType::SessionId,
    )
    .projection_expression([SESSION_ID, SESSION_EXPIRY, EMAIL].join(", "))
    .send()
    .await
    .map_err(aws_sdk_dynamodb::Error::from);

    match query {
        Ok(o) => {
            let items = o.items.ok_or_else(|| UNHANDLED)?;
            let item_in_query = items.first().ok_or_else(|| {
                log::error!("Unable to get first item in check_if_session_is_valid query");
                UNHANDLED
            })?;
            let session_id = item_in_query
                .get(SESSION_ID)
                .ok_or_else(|| {
                    log::error!("Unable to get session_id in check_if_session_is_valid query");
                    UNHANDLED
                })?
                .as_s()
                .map_err(|e| {
                    log::error!(
                        "Can't get session_id as string in check_if_session_is_valid query{:?}",
                        e
                    );
                    UNHANDLED
                })?;
            let session_expiry = item_in_query.get(SESSION_EXPIRY).ok_or_else(|| {
                    log::error!("Unable to get session_id in check_if_session_is_valid query");
                    UNHANDLED
                })?.as_n().map_err(|e| {
                    log::error!("Can't get session_expiry as number in check_if_session_is_valid query {:?}", e);
                    UNHANDLED
                })?.parse::<i64>().map_err(|e| {
                    log::error!("Could not parse string as i64 {:?}", e);
                    UNHANDLED
                })?;
            let email = item_in_query
                .get(EMAIL)
                .ok_or_else(|| {
                    log::error!("Unable to get email in check_if_session_is_valid query");
                    UNHANDLED
                })?
                .as_s()
                .map_err(|e| {
                    log::error!(
                        "Can't get email as string in check_if_session_is_valid query {:?}",
                        e
                    );
                    UNHANDLED
                })?;
            let now = Utc::now().timestamp();
            Ok((
                *session_id == session_id_cookie && now < session_expiry,
                email.to_owned(),
            ))
        }
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }
}

pub fn extract_email_from_query(o: &QueryOutput) -> Result<String, ServerFnError<NexusError>> {
    let items: Vec<HashMap<String, AttributeValue>> = o.items.clone().ok_or_else(|| {
        log::error!("Unable to get items from QueryOutput in extract_email_from_query");
        UNHANDLED
    })?;
    let item = items
        .first()
        .ok_or_else(|| {
            log::error!("Cannot get first item in extract_email_from_query");
            UNHANDLED
        })?
        .clone();
    let email_string = item
        .get(table_attributes::EMAIL)
        .ok_or_else(|| {
            log::error!("Unable to find email attribute (should be impossible)");
            UNHANDLED
        })?
        .as_s()
        .map_err(|e| {
            log::error!("Could not get email as string {:?}", e);
            UNHANDLED
        })?
        .clone();

    Ok(email_string)
}

pub fn extract_email_verification_request_time_from_query(
    o: &QueryOutput,
) -> Result<i64, ServerFnError<NexusError>> {
    let items: Vec<HashMap<String, AttributeValue>> = o.items.clone().ok_or_else(|| {
        log::error!("Unable to get items from QueryOutput in extract_email_verification_request_time_from_query");
        UNHANDLED
    })?;
    let item = items
        .first()
        .ok_or_else(|| {
            log::error!(
                "Cannot get first item in extract_email_verification_request_time_from_query"
            );
            UNHANDLED
        })?
        .clone();
    let email_verification_request_time = item
        .get(table_attributes::EMAIL_VERIFICATION_REQUEST_TIME)
        .ok_or_else(|| {
            log::error!("Unable to find email verification request time attribute (should be impossible)");
            UNHANDLED
        })?
        .as_n()
        .map_err(|e| {
            log::error!("Could not get email verification request time as number {:?}", e);
            UNHANDLED
        })?
        .parse::<i64>()
        .map_err(|e| {
            log::error!("Could not parse string into i64 in extract_email_verification_request_time_from_query {:?}", e);
            UNHANDLED
        })?;
    Ok(email_verification_request_time)
}

pub fn extract_id_from_query(o: QueryOutput) -> Result<String, ServerFnError<NexusError>> {
    let items = o.items.ok_or_else(|| UNHANDLED)?.clone();
    let item = items
        .first()
        .ok_or_else(|| {
            log::error!("Cannot get first item in extract_id_from_query");
            UNHANDLED
        })?
        .clone();
    let user_uuid = item
        .get(table_attributes::USER_UUID)
        .ok_or_else(|| {
            log::error!("Unable to find user_uuid attribute (should be impossible)");
            UNHANDLED
        })?
        .as_s()
        .map_err(|e| {
            log::error!("Could not get user_uuid as string {:?}", e);
            UNHANDLED
        })?
        .clone();

    Ok(user_uuid)
}

pub async fn check_email_uniqueness(
    email: String,
    client: &aws_sdk_dynamodb::Client,
) -> Result<bool, ServerFnError<NexusError>> {
    let db_query = client
        .get_item()
        .table_name(get_table_name())
        .key(EMAIL, AttributeValue::S(email))
        .projection_expression([EMAIL].join(", "))
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);

    match db_query {
        Ok(o) => {
            let items = o.item.ok_or_else(|| {
                log::error!("Could not get item from items in check_new_email_uniqueness");
                UNHANDLED
            })?;
            let item = items.get(EMAIL);
            Ok(item.is_none())
        }
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }
}

pub async fn get_email_from_session_id(
    session_id_cookie: String,
    client: &aws_sdk_dynamodb::Client,
) -> Result<String, ServerFnError<NexusError>> {
    let db_query_result = query_setup(client, session_id_cookie, TableKeyType::SessionId)
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);

    let email = match db_query_result {
        Ok(o) => Ok(extract_email_from_query(&o)?),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }?;

    Ok(email)
}

pub fn handle_dynamo_generic_error(e: aws_sdk_dynamodb::Error) -> ServerFnError<NexusError> {
    log::error!("{:?}", e);
    ServerFnError::from(NexusError::GenericDynamoServiceError)
}
