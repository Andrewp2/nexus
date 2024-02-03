use std::collections::HashMap;

use argon2::{
    password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use aws_sdk_dynamodb::{
    operation::{
        get_item::GetItemError,
        query::{QueryError, QueryOutput},
    },
    types::AttributeValue,
    Client as DynamoClient,
};
use aws_sdk_ses::Client as SesClient;
use axum_extra::extract::CookieJar;
use chrono::Utc;
use leptos::{use_context, ServerFnError};
use leptos_axum::extract;
use rand::rngs::OsRng;
use stripe::Client as StripeClient;

use crate::{
    dynamo::constants::{
        index,
        table_attributes::{self, EMAIL, SESSION_EXPIRY, SESSION_ID},
    },
    env_var::get_table_name,
    errors::NexusError,
};

pub fn dynamo_client() -> Result<DynamoClient, ServerFnError> {
    use_context::<DynamoClient>().ok_or_else(|| ServerFnError::new(NexusError::Unhandled))
}

pub fn ses_client() -> Result<SesClient, ServerFnError> {
    use_context::<SesClient>().ok_or_else(|| ServerFnError::new(NexusError::Unhandled))
}

pub fn stripe_client() -> Result<StripeClient, ServerFnError> {
    use_context::<StripeClient>().ok_or_else(|| ServerFnError::new(NexusError::Unhandled))
}

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password: &str, database_hash: &str) -> bool {
    if let Ok(hash) = PasswordHash::new(&database_hash) {
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

pub async fn get_session_cookie() -> Result<String, ServerFnError> {
    let cookie_jar: CookieJar = extract().await?;
    let session_id = cookie_jar.get(SESSION_ID).ok_or_else(|| {
        log::error!("Couldn't get session_id from cookie_jar");
        ServerFnError::new(NexusError::Unhandled)
    })?;
    let f = session_id
        .value()
        .to_string()
        .strip_prefix("__Host-")
        .ok_or_else(|| {
            log::error!("Couldn't remove __Host- prefix from cookie");
            ServerFnError::new(NexusError::Unhandled)
        })?
        .to_string();

    return Ok(f);
    // Ok(extract(|cookie_jar: CookieJar| async move { cookie_jar })
    //     .await
    //     .map_err(|e| {
    //         log::error!("Couldn't extract cookie_jar {:?}", e);
    //         ServerFnError::new(NexusError::Unhandled)
    //     })?
    //     .get(SESSION_ID)
    //     .ok_or_else(|| {
    //         log::error!("Couldn't get session_id from cookie_jar");
    //         ServerFnError::new(NexusError::Unhandled)
    //     })?
    //     .value()
    //     .to_string()
    //     .strip_prefix("__Host-")
    //     .ok_or_else(|| {
    //         log::error!("Couldn't remove __Host- prefix from cookie");
    //         ServerFnError::new(NexusError::Unhandled)
    //     })?
    //     .to_owned())
}

pub async fn check_if_session_is_valid(
    session_id_cookie: String,
    client: &aws_sdk_dynamodb::Client,
) -> Result<(bool, String), ServerFnError> {
    let query = client
        .query()
        .table_name(get_table_name())
        .limit(1)
        .index_name(crate::dynamo::constants::index::SESSION_ID)
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k", SESSION_ID)
        .expression_attribute_names(":v", session_id_cookie.clone())
        .projection_expression([SESSION_ID, SESSION_EXPIRY, EMAIL].join(", "))
        .send()
        .await;

    match query {
        Ok(o) => {
            let items = o
                .items
                .ok_or_else(|| ServerFnError::new(NexusError::Unhandled))?;
            let item_in_query = items.first().ok_or_else(|| {
                log::error!("Unable to get first item in check_if_session_is_valid query");
                ServerFnError::new(NexusError::Unhandled)
            })?;
            let session_id = item_in_query
                .get(SESSION_ID)
                .ok_or_else(|| {
                    log::error!("Unable to get session_id in check_if_session_is_valid query");
                    ServerFnError::new(NexusError::Unhandled)
                })?
                .as_s()
                .map_err(|e| {
                    log::error!(
                        "Can't get session_id as string in check_if_session_is_valid query{:?}",
                        e
                    );
                    ServerFnError::new(NexusError::Unhandled)
                })?;
            let session_expiry = item_in_query.get(SESSION_EXPIRY).ok_or_else(|| {
                    log::error!("Unable to get session_id in check_if_session_is_valid query");
                    ServerFnError::new(NexusError::Unhandled)
                })?.as_n().map_err(|e| {
                    log::error!("Can't get session_expiry as number in check_if_session_is_valid query {:?}", e);
                    ServerFnError::new(NexusError::Unhandled)
                })?.parse::<i64>().map_err(|e| {
                    log::error!("Could not parse string as i64 {:?}", e);
                    ServerFnError::new(NexusError::Unhandled)
                })?;
            let email = item_in_query
                .get(EMAIL)
                .ok_or_else(|| {
                    log::error!("Unable to get email in check_if_session_is_valid query");
                    ServerFnError::new(NexusError::Unhandled)
                })?
                .as_s()
                .map_err(|e| {
                    log::error!(
                        "Can't get email as string in check_if_session_is_valid query {:?}",
                        e
                    );
                    ServerFnError::new(NexusError::Unhandled)
                })?;
            let now = Utc::now().timestamp();
            Ok((
                *session_id == session_id_cookie && now < session_expiry,
                email.to_owned(),
            ))
        }
        Err(e) => Err(ServerFnError::new(match e.into_service_error() {
            QueryError::InternalServerError(e2) => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
            QueryError::InvalidEndpointException(e2) => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
            QueryError::ProvisionedThroughputExceededException(e2) => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
            QueryError::RequestLimitExceeded(e2) => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
            QueryError::ResourceNotFoundException(e2) => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
            e2 => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
        })),
    }
}

pub fn extract_email_from_query(o: QueryOutput) -> Result<String, ServerFnError> {
    let items: Vec<HashMap<String, AttributeValue>> = o.items.ok_or_else(|| {
        log::error!("Unable to get items from QueryOutput in extract_email_from_query");
        ServerFnError::new(NexusError::Unhandled)
    })?;
    let item = items
        .first()
        .ok_or_else(|| {
            log::error!("Cannot get first item in extract_email_from_query");
            ServerFnError::new(NexusError::Unhandled)
        })?
        .clone();
    let email_string = item
        .get(table_attributes::EMAIL)
        .ok_or_else(|| {
            log::error!("Unable to find email attribute (should be impossible)");
            ServerFnError::new(NexusError::Unhandled)
        })?
        .as_s()
        .map_err(|e| {
            log::error!("Could not get email as string {:?}", e);
            ServerFnError::new(NexusError::Unhandled)
        })?
        .clone();

    Ok(email_string)
}

pub async fn check_email_uniqueness(
    email: String,
    client: &aws_sdk_dynamodb::Client,
) -> Result<bool, ServerFnError> {
    let db_query = client
        .get_item()
        .table_name(get_table_name())
        .key(EMAIL, AttributeValue::S(email))
        .projection_expression([EMAIL].join(", "))
        .send()
        .await;

    match db_query {
        Ok(o) => {
            let items = o.item.ok_or_else(|| {
                log::error!("Could not get item from items in check_new_email_uniqueness");
                ServerFnError::new(NexusError::Unhandled)
            })?;
            let item = items.get(EMAIL);
            Ok(item.is_none())
        }
        Err(e) => match e.into_service_error() {
            GetItemError::InternalServerError(e2) => {
                log::error!("check_new_email_uniqueness error {:?}", e2);
                Err(ServerFnError::new(NexusError::Unhandled))
            }
            GetItemError::InvalidEndpointException(e2) => {
                log::error!("check_new_email_uniqueness error {:?}", e2);
                Err(ServerFnError::new(NexusError::Unhandled))
            }
            GetItemError::ProvisionedThroughputExceededException(e2) => {
                log::error!("check_new_email_uniqueness error {:?}", e2);
                Err(ServerFnError::new(NexusError::Unhandled))
            }
            GetItemError::RequestLimitExceeded(e2) => {
                log::error!("check_new_email_uniqueness error {:?}", e2);
                Err(ServerFnError::new(NexusError::Unhandled))
            }
            GetItemError::ResourceNotFoundException(e2) => {
                log::error!("check_new_email_uniqueness error {:?}", e2);
                Err(ServerFnError::new(NexusError::Unhandled))
            }
            e2 => {
                log::error!("check_new_email_uniqueness error {:?}", e2);
                Err(ServerFnError::new(NexusError::Unhandled))
            }
        },
    }
}

pub async fn get_email_from_session_id(
    session_id_cookie: String,
    client: &aws_sdk_dynamodb::Client,
) -> Result<String, ServerFnError> {
    let db_query_result = client
        .query()
        .limit(1)
        .table_name(get_table_name())
        .index_name(index::SESSION_ID)
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k".to_string(), table_attributes::SESSION_ID)
        .expression_attribute_values(":v".to_string(), AttributeValue::S(session_id_cookie))
        .send()
        .await;

    let email = match db_query_result {
        Ok(o) => Ok(extract_email_from_query(o)?),
        Err(e) => Err(ServerFnError::new(match e.into_service_error() {
            QueryError::InternalServerError(e2) => {
                log::error!("get_email_from_session_id {:?}", e2);
                NexusError::Unhandled
            }
            QueryError::InvalidEndpointException(e2) => {
                log::error!("get_email_from_session_id {:?}", e2);
                NexusError::Unhandled
            }
            QueryError::ProvisionedThroughputExceededException(e2) => {
                log::error!("get_email_from_session_id {:?}", e2);
                NexusError::Unhandled
            }
            QueryError::RequestLimitExceeded(e2) => {
                log::error!("get_email_from_session_id {:?}", e2);
                NexusError::Unhandled
            }
            QueryError::ResourceNotFoundException(e2) => {
                log::error!("get_email_from_session_id {:?}", e2);
                NexusError::Unhandled
            }
            e2 => {
                log::error!("get_email_from_session_id {:?}", e2);
                NexusError::Unhandled
            }
        })),
    }?;

    Ok(email)
}

