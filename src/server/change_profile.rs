use super::utilities::{dynamo_client, get_email_from_session_id, get_session_cookie, ses_client};
use crate::{
    dynamo::constants::{
        get_table_name,
        index::EMAIL_VERIFICATION_UUID,
        table_attributes::{self, EMAIL, EMAIL_VERIFIED, SESSION_EXPIRY, SESSION_ID},
    },
    errors::NexusError,
    site::constants::{SITE_DOMAIN, SITE_EMAIL_ADDRESS, SITE_FULL_DOMAIN},
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
use rustrict::{Censor, Type};
use uuid::Uuid;

/// Starts a request to change email
pub async fn change_email_request(new_email: String) -> Result<(), ServerFnError> {
    if !EmailAddress::is_valid(&new_email) {
        return Err(ServerFnError::new(NexusError::BadEmailAddress));
    }
    let session_id_cookie = get_session_cookie().await?;
    let client = dynamo_client()?;
    let old_user_query = client
        .query()
        .table_name(get_table_name())
        .limit(1)
        .index_name(crate::dynamo::constants::index::SESSION_ID)
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k", SESSION_ID)
        .expression_attribute_names(":v", session_id_cookie.clone())
        .send()
        .await;
    let user = match old_user_query {
        Ok(o) => Ok(o),
        Err(e) => Err(ServerFnError::new(match e.into_service_error() {
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
        })),
    }?;
    let items = user.items.ok_or_else(|| {
        log::error!("Could not get items from user");
        ServerFnError::new(NexusError::Unhandled)
    })?;
    let attributes = items.first().ok_or_else(|| {
        log::error!("Could not get first item from items");
        ServerFnError::new(NexusError::Unhandled)
    })?;
    let session_expiry = attributes
        .get(SESSION_EXPIRY)
        .ok_or_else(|| {
            log::error!("Could not get session expiry");
            ServerFnError::new(NexusError::Unhandled)
        })?
        .as_n()
        .map_err(|e| {
            log::error!("Could not convert session expiry to number {:?}", e);
            ServerFnError::new(NexusError::Unhandled)
        })?
        .parse::<i64>()
        .map_err(|e| {
            log::error!("Could not parse sesion expiry number as i64 {:?}", e);
            ServerFnError::new(NexusError::Unhandled)
        })?;
    let now = Utc::now().timestamp();
    if now >= session_expiry {
        return Err(ServerFnError::new(NexusError::InvalidSession));
    }
    let check_email_not_already_exists_expression =
        format!("attribute_not_exists({})", table_attributes::EMAIL);
    // this involves a query for the new email, but we could just use a conditional put
    let mut put = client
        .put_item()
        .table_name(get_table_name())
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
        Err(e) => Err(ServerFnError::new(
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
        .data(format!(
            "[{}] Please verify your email address",
            SITE_DOMAIN
        ))
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
        Err(e) => Err(handle_send_email_error(e, new_email)),
    }?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

fn handle_send_email_error(e: SdkError<SendEmailError>, new_email: String) -> ServerFnError {
    log::error!("Warning, we created a new account for {} but we weren't able to send them the verification email!", new_email);
    ServerFnError::new(match e.into_service_error() {
        SendEmailError::AccountSendingPausedException(e2) => {
            log::error!("handle_verify_change_email_request_error {:?}", e2);
            NexusError::Unhandled
        }
        SendEmailError::ConfigurationSetDoesNotExistException(e2) => {
            log::error!("handle_verify_change_email_request_error {:?}", e2);
            NexusError::Unhandled
        }
        SendEmailError::ConfigurationSetSendingPausedException(e2) => {
            log::error!("handle_verify_change_email_request_error {:?}", e2);
            NexusError::Unhandled
        }
        SendEmailError::MailFromDomainNotVerifiedException(e2) => {
            log::error!("handle_verify_change_email_request_error {:?}", e2);
            NexusError::Unhandled
        }
        SendEmailError::MessageRejected(e2) => {
            log::error!("handle_verify_change_email_request_error {:?}", e2);
            NexusError::Unhandled
        }
        e2 => {
            log::error!("handle_verify_change_email_request_error {:?}", e2);
            NexusError::Unhandled
        }
    })
}

async fn change_value(name: &str, value: AttributeValue) -> Result<(), ServerFnError> {
    let client = dynamo_client()?;
    let session_id = get_session_cookie().await?;
    let email = get_email_from_session_id(session_id, &client).await?;
    let update_resp = client
        .update_item()
        .table_name(get_table_name())
        .key(table_attributes::EMAIL, AttributeValue::S(email))
        .update_expression("SET #e = :r")
        .expression_attribute_names("e".to_string(), name)
        .expression_attribute_values(":r", value)
        .send()
        .await;
    match update_resp {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::new(match e.into_service_error() {
            UpdateItemError::ConditionalCheckFailedException(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::InternalServerError(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::InvalidEndpointException(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::ItemCollectionSizeLimitExceededException(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::ProvisionedThroughputExceededException(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::RequestLimitExceeded(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::ResourceNotFoundException(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            UpdateItemError::TransactionConflictException(e2) => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
            e2 => {
                log::error!("{:?}", e2);
                NexusError::Unhandled
            }
        })),
    }
}

pub async fn change_display_name(new_display_name: String) -> Result<(), ServerFnError> {
    let mut censor = Censor::from_str(&new_display_name);
    let censor_type = censor.analyze();
    if censor_type.is(Type::MODERATE_OR_HIGHER) {
        return Err(ServerFnError::new(NexusError::DisplayNameInappropriate));
    }
    let display_name_av = AttributeValue::S(new_display_name);
    change_value(table_attributes::DISPLAY_NAME, display_name_av).await
}

pub async fn change_password(new_password: String) -> Result<(), ServerFnError> {
    let new_password_av = AttributeValue::S(new_password);
    change_value(table_attributes::PASSWORD, new_password_av).await
}

