use aws_sdk_dynamodb::{types::AttributeValue, Client};
use leptos::ServerFnError;

use crate::{dynamo::constants::table_attributes, env_var::get_table_name, errors::NexusError};

use super::utilities::{
    dynamo_client, get_email_from_session_id, get_session_cookie, handle_dynamo_generic_error,
};

pub async fn logout() -> Result<(), ServerFnError<NexusError>> {
    let client = dynamo_client()?;
    let session_id_cookie = get_session_cookie().await?;
    let email = get_email_from_session_id(session_id_cookie, &client).await?;
    set_expiry_for_email(email, &client).await
}

async fn set_expiry_for_email(
    email: String,
    client: &Client,
) -> Result<(), ServerFnError<NexusError>> {
    let db_update_result = client
        .update_item()
        .table_name(get_table_name())
        .key(table_attributes::EMAIL, AttributeValue::S(email))
        .update_expression("SET #e = :r")
        .expression_attribute_names("e".to_string(), table_attributes::SESSION_EXPIRY)
        .expression_attribute_values(":r", AttributeValue::N("0".to_string()))
        .send()
        .await
        .map_err(|e| aws_sdk_dynamodb::Error::from(e));

    match db_update_result {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }
}
