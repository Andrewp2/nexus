use super::utilities::{dynamo_client, extract_email_from_query, ses_client};
use crate::{
    dynamo::constants::*,
    errors::NexusError,
    site::constants::{SITE_DOMAIN, SITE_EMAIL_ADDRESS, SITE_FULL_DOMAIN},
};
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::{query::QueryError, update_item::UpdateItemError},
    types::AttributeValue,
};
use aws_sdk_ses::{
    operation::send_email::SendEmailError,
    types::{Body, Content, Destination, Message},
};
use leptos::ServerFnError;

/// Sends an email to the given users address with a link to verify their account.
pub async fn send_verification_email(
    email_address: String,
    verification_uuid: String,
) -> Result<(), ServerFnError> {
    let ses_client = ses_client()?;
    let body = format!(
        "Hello,
Somebody just used this email address to sign up at {}.
        
If this was you, verify your email by clicking on the link below:
        
https://{}/email-verification/{}
        
If this was not you, you may ignore this email.",
        SITE_DOMAIN,
        SITE_FULL_DOMAIN,
        verification_uuid.to_string()
    );
    let email_body_html = Content::builder().data(body).build()?;
    let email_body = Body::builder().html(email_body_html).build();
    let email_subject_content = Content::builder()
        .data("[MySite] Please verify your email address")
        .build()?;
    let email_message = Message::builder()
        .subject(email_subject_content)
        .body(email_body)
        .build();
    let email_send_resp = ses_client
        .send_email()
        .source(SITE_EMAIL_ADDRESS)
        .destination(Destination::builder().to_addresses(email_address).build())
        .message(email_message)
        .send()
        .await;
    match email_send_resp {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_send_email_error(e)),
    }?;
    Ok(())
}

fn handle_send_email_error(e: SdkError<SendEmailError>) -> ServerFnError {
    ServerFnError::ServerError(
        match e.into_service_error() {
            SendEmailError::AccountSendingPausedException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericSesError
            }
            SendEmailError::ConfigurationSetDoesNotExistException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericSesError
            }
            SendEmailError::ConfigurationSetSendingPausedException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericSesError
            }
            SendEmailError::MailFromDomainNotVerifiedException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericSesError
            }
            SendEmailError::MessageRejected(e) => {
                log::error!("{:?}", e);
                NexusError::GenericSesError
            }
            e => {
                log::error!("{:?}", e);
                NexusError::GenericSesError
            }
        }
        .to_string(),
    )
}

/// Verifies a given email_uuid
pub async fn verify_email_for_signup(email_uuid: String) -> Result<(), ServerFnError> {
    let client = dynamo_client()?;
    // first we have to query to find the email address associated with this verification attempt.
    let db_query_result = client
        .query()
        .limit(1)
        .table_name(TABLE_NAME)
        .index_name(index::EMAIL_VERIFICATION_UUID)
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k".to_string(), table_attributes::EMAIL_VERIFICATION_UUID)
        .expression_attribute_values(":v".to_string(), AttributeValue::S(email_uuid))
        .send()
        .await;

    let email = match db_query_result {
        Ok(o) => Ok(extract_email_from_query(o)?),
        Err(e) => Err(handle_email_query_error(e)),
    }?;

    // secondly if we can find the email, update its verification field
    let db_update_result = client
        .update_item()
        .table_name(TABLE_NAME)
        .key(
            table_attributes::EMAIL,
            AttributeValue::S(email.to_string()),
        )
        .update_expression("SET #e = :r")
        .expression_attribute_names("e".to_string(), table_attributes::EMAIL_VERIFIED)
        .expression_attribute_values(":r", AttributeValue::Bool(true))
        .send()
        .await;

    match db_update_result {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_verify_email_update_error(e)),
    }
}

fn handle_email_query_error(e: aws_sdk_dynamodb::error::SdkError<QueryError>) -> ServerFnError {
    ServerFnError::ServerError(
        match e.into_service_error() {
            QueryError::InternalServerError(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            QueryError::InvalidEndpointException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            QueryError::ProvisionedThroughputExceededException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            QueryError::RequestLimitExceeded(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            QueryError::ResourceNotFoundException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            e => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
        }
        .to_string(),
    )
}

fn handle_verify_email_update_error(e: SdkError<UpdateItemError>) -> ServerFnError {
    ServerFnError::ServerError(
        match e.into_service_error() {
            UpdateItemError::ConditionalCheckFailedException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
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
        }
        .to_string(),
    )
}

