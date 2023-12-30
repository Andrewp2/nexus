use aws_sdk_dynamodb::{
    operation::{query::QueryError, update_item::UpdateItemError},
    types::AttributeValue,
    Client,
};
use leptos::ServerFnError;

use crate::{
    dynamo::constants::{index, table_attributes, TABLE_NAME},
    errors::NexusError,
};

use super::utilities::{dynamo_client, extract_email_from_query, get_session_cookie};

pub async fn logout() -> Result<(), ServerFnError> {
    let client = dynamo_client()?;
    let session_id_cookie = get_session_cookie().await?;
    let email = get_email_from_session_id(session_id_cookie, &client).await?;
    set_expiry_for_email(email, &client).await
}

async fn set_expiry_for_email(email: String, client: &Client) -> Result<(), ServerFnError> {
    let db_update_result = client
        .update_item()
        .table_name(TABLE_NAME)
        .key(table_attributes::EMAIL, AttributeValue::S(email))
        .update_expression("SET #e = :r")
        .expression_attribute_names("e".to_string(), table_attributes::SESSION_EXPIRY)
        .expression_attribute_values(":r", AttributeValue::N("0".to_string()))
        .send()
        .await;

    match db_update_result {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                UpdateItemError::ConditionalCheckFailedException(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::InternalServerError(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::InvalidEndpointException(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::ItemCollectionSizeLimitExceededException(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::ProvisionedThroughputExceededException(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::RequestLimitExceeded(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::ResourceNotFoundException(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                UpdateItemError::TransactionConflictException(e2) => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
                e2 => {
                    log::error!("set_expiry_for_mail Unexpected Error: {:?}", e2);
                    NexusError::Unhandled.to_string()
                }
            }
            .to_string(),
        )),
    }
}

async fn get_email_from_session_id(
    session_id_cookie: String,
    client: &Client,
) -> Result<String, ServerFnError> {
    let db_query_result = client
        .query()
        .limit(1)
        .table_name(TABLE_NAME)
        .index_name(index::SESSION_ID)
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k".to_string(), table_attributes::SESSION_ID)
        .expression_attribute_values(":v".to_string(), AttributeValue::S(session_id_cookie))
        .send()
        .await;

    let email = match db_query_result {
        Ok(o) => Ok(extract_email_from_query(o)?),
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                QueryError::InternalServerError(e2) => {
                    log::error!("get_email_from_session_id {:?}", e2);
                    NexusError::Unhandled
                }
                QueryError::InvalidEndpointException(e2) => {
                    log::error!("get_email_from_session_id");
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
            }
            .to_string(),
        )),
    }?;

    Ok(email)
}

