use super::utilities::{dynamo_client, extract_email_from_query, get_session_cookie, ses_client};
use crate::{
    dynamo::constants::{
        index::{self, EMAIL_VERIFICATION_UUID},
        table_attributes::{self, EMAIL, EMAIL_VERIFIED, SESSION_EXPIRY, SESSION_ID},
        TABLE_NAME,
    },
    errors::NexusError,
    site::constants::{SITE_EMAIL_ADDRESS, SITE_FULL_DOMAIN},
};
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::{put_item::PutItemError, query::QueryError, update_item::UpdateItemError},
    types::AttributeValue,
};
use aws_sdk_ses::{
    operation::send_email::SendEmailError,
    types::{Body, Content, Destination, Message},
};
use chrono::Utc;
use email_address::EmailAddress;
use leptos::ServerFnError;
use uuid::Uuid;

/// Starts a request to change email
pub async fn change_email_request(new_email: String) -> Result<(), ServerFnError> {
    if !EmailAddress::is_valid(&new_email) {
        return Err(ServerFnError::ServerError(
            NexusError::BadEmailAddress.to_string(),
        ));
    }
    let session_id_cookie = get_session_cookie().await?;
    let client = dynamo_client()?;
    let old_user_query = client
        .query()
        .table_name(TABLE_NAME)
        .limit(1)
        .index_name(crate::dynamo::constants::index::SESSION_ID)
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k", SESSION_ID)
        .expression_attribute_names(":v", session_id_cookie.clone())
        .send()
        .await;
    let user = match old_user_query {
        Ok(o) => Ok(o),
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                QueryError::InternalServerError(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                QueryError::InvalidEndpointException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                QueryError::ProvisionedThroughputExceededException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                QueryError::RequestLimitExceeded(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                QueryError::ResourceNotFoundException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                e2 => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
            }
            .to_string(),
        )),
    }?;
    let items = user.items.ok_or_else(|| {
        log::error!("Could not get items from user");
        ServerFnError::ServerError(NexusError::Unhandled.to_string())
    })?;
    let attributes = items.first().ok_or_else(|| {
        log::error!("Could not get first item from items");
        ServerFnError::ServerError(NexusError::Unhandled.to_string())
    })?;
    let session_expiry = attributes
        .get(SESSION_EXPIRY)
        .unwrap()
        .as_n()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    let now = Utc::now().timestamp();
    if now >= session_expiry {
        return Err(ServerFnError::ServerError(
            NexusError::InvalidSession.to_string(),
        ));
    }
    let check_email_not_already_exists_expression =
        format!("attribute_not_exists({})", table_attributes::EMAIL);
    // this involves a query for the new email, but we could just use a conditional put
    let mut put = client
        .put_item()
        .table_name(TABLE_NAME)
        .condition_expression(check_email_not_already_exists_expression);

    for (name, attribute) in attributes.iter() {
        put = put.item(name, attribute.clone());
    }
    let change_email_verification_uuid = Uuid::new_v4().to_string();
    put = put.item(
        EMAIL_VERIFICATION_UUID,
        AttributeValue::S(change_email_verification_uuid.clone()),
    );
    put = put.item(EMAIL_VERIFIED, AttributeValue::Bool(false));
    let put_resp = put.send().await;

    match put_resp {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                PutItemError::ConditionalCheckFailedException(e2) => {
                    log::warn!("{:?}", e2);
                    NexusError::EmailAlreadyInUse
                }
                PutItemError::InvalidEndpointException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                PutItemError::ItemCollectionSizeLimitExceededException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                PutItemError::ProvisionedThroughputExceededException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                PutItemError::RequestLimitExceeded(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                PutItemError::ResourceNotFoundException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                PutItemError::TransactionConflictException(e2) => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
                e2 => {
                    log::error!("{:?}", e2);
                    NexusError::Unhandled
                }
            }
            .to_string(),
        )),
    }?;
    let old_email = attributes.get(EMAIL).unwrap().as_s().unwrap();
    let ses_client = ses_client()?;
    let body = format!(
        "Hello,
Did you just request that your email address be changed?
If so, click on the below link to accept the email change:

https://{}/email-verification/{}

If you did not request an email address change, please change your password.",
        SITE_FULL_DOMAIN,
        change_email_verification_uuid.to_string()
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
        .destination(Destination::builder().to_addresses(old_email).build())
        .message(email_message)
        .send()
        .await;
    match email_send_resp {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_verify_change_email_request_error(e, new_email)),
    }?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

fn handle_verify_change_email_request_error(
    e: SdkError<SendEmailError>,
    new_email: String,
) -> ServerFnError {
    log::error!("Warning, we created a new account for {} but we weren't able to send them the verification email!", new_email);
    ServerFnError::ServerError(
        match e.into_service_error() {
            SendEmailError::AccountSendingPausedException(e2) => {
                log::error!("handle_verify_change_email_request_error {:?}", e2);
                NexusError::Unhandled.to_string()
            }
            SendEmailError::ConfigurationSetDoesNotExistException(e2) => {
                log::error!("handle_verify_change_email_request_error {:?}", e2);
                NexusError::Unhandled.to_string()
            }
            SendEmailError::ConfigurationSetSendingPausedException(e2) => {
                log::error!("handle_verify_change_email_request_error {:?}", e2);
                NexusError::Unhandled.to_string()
            }
            SendEmailError::MailFromDomainNotVerifiedException(e2) => {
                log::error!("handle_verify_change_email_request_error {:?}", e2);
                NexusError::Unhandled.to_string()
            }
            SendEmailError::MessageRejected(e2) => {
                log::error!("handle_verify_change_email_request_error {:?}", e2);
                NexusError::Unhandled.to_string()
            }
            e2 => {
                log::error!("handle_verify_change_email_request_error {:?}", e2);
                NexusError::Unhandled.to_string()
            }
        }
        .to_string(),
    )
}

pub async fn change_email_validation(email_uuid: String) -> Result<(), ServerFnError> {
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
        Err(e) => Err(ServerFnError::ServerError(
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
        )),
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

pub async fn change_display_name(new_display_name: String) -> Result<(), ServerFnError> {
    // update old record
    Ok(())
}

pub async fn change_password(new_password: String) -> Result<(), ServerFnError> {
    // update old record
    Ok(())
}

