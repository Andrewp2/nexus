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

use super::utilities::{
    dynamo_client, extract_email_from_query, get_email_from_session_id, get_session_cookie,
};

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

