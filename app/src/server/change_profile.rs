use super::{
    csrf::{self, validate_csrf_header},
    globals::{
        dynamo::{
            self,
            constants::table_attributes::{
                self, EMAIL, EMAIL_VERIFICATION_REQUEST_TIME, EMAIL_VERIFICATION_UUID,
                EMAIL_VERIFIED, SESSION_EXPIRY,
            },
            query_setup, update_setup,
        },
        env_var::get_table_name,
    },
    utilities::{
        dynamo_client, get_email_from_session_id, get_session_cookie, handle_dynamo_generic_error,
        kms_client, ses_client,
    },
};
use crate::{
    errors::{NexusError, UNHANDLED},
    site::constants::{SITE_DOMAIN, SITE_EMAIL_ADDRESS, SITE_FULL_DOMAIN},
};
use aws_sdk_dynamodb::{
    operation::query::QueryOutput, types::AttributeValue, Client as DynamoClient,
};
use aws_sdk_ses::types::{Body, Content, Destination, Message};
use chrono::Utc;
use email_address::EmailAddress;
use leptos::ServerFnError;
use rustrict::{Censor, Type};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Starts a request to change email
pub async fn change_email_request(new_email: String) -> Result<(), ServerFnError<NexusError>> {
    if !EmailAddress::is_valid(&new_email) {
        return Err(ServerFnError::from(NexusError::BadEmailAddress));
    }
    let session_id_cookie = get_session_cookie().await?;
    let client = dynamo_client()?;
    let kms_client = kms_client()?;
    let x: &aws_sdk_kms::Client = &kms_client;
    if !validate_csrf_header(x, session_id_cookie.clone()).await? {
        return Err(UNHANDLED);
    }
    let old_user_query = query_setup(
        &client,
        session_id_cookie.clone(),
        dynamo::TableKeyType::SessionId,
    )
    .send()
    .await
    .map_err(aws_sdk_dynamodb::Error::from);
    let attributes = get_attributes_from_query(old_user_query)?;
    let session_expiry = get_session_expiry_from_attributes(&attributes)?;
    let now = Utc::now().timestamp();
    if now >= session_expiry {
        return Err(ServerFnError::from(NexusError::InvalidSession));
    }
    let old_email = get_old_email_from_request(&attributes)?;
    let change_email_verification_uuid =
        put_new_email_if_it_doesnt_exist(client, &attributes, now).await?;
    send_email_for_change_email_request(change_email_verification_uuid, old_email, new_email)
        .await?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

fn get_session_expiry_from_attributes(
    attributes: &HashMap<String, AttributeValue>,
) -> Result<i64, ServerFnError<NexusError>> {
    let session_expiry = attributes
        .get(SESSION_EXPIRY)
        .ok_or_else(|| {
            log::error!("Could not get session expiry");
            UNHANDLED
        })?
        .as_n()
        .map_err(|e| {
            log::error!("Could not convert session expiry to number {:?}", e);
            UNHANDLED
        })?
        .parse::<i64>()
        .map_err(|e| {
            log::error!("Could not parse sesion expiry number as i64 {:?}", e);
            UNHANDLED
        })?;
    Ok(session_expiry)
}

fn get_attributes_from_query(
    old_user_query: Result<QueryOutput, aws_sdk_dynamodb::Error>,
) -> Result<HashMap<String, AttributeValue>, ServerFnError<NexusError>> {
    let user = match old_user_query {
        Ok(o) => Ok(o),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }?;
    let items = user.items.ok_or_else(|| {
        log::error!("Could not get items from user");
        UNHANDLED
    })?;
    let attributes = items.first().ok_or_else(|| {
        log::error!("Could not get first item from items");
        UNHANDLED
    })?;
    Ok(attributes.clone())
}

fn get_old_email_from_request(
    attributes: &HashMap<String, AttributeValue>,
) -> Result<&String, ServerFnError<NexusError>> {
    let old_email = attributes
        .get(EMAIL)
        .ok_or_else(|| {
            log::error!("Could not get email attribute");
            UNHANDLED
        })?
        .as_s()
        .map_err(|e| {
            log::error!("Could not get email attribute as string {:?}", e);
            UNHANDLED
        })?;
    Ok(old_email)
}

async fn put_new_email_if_it_doesnt_exist(
    client: Arc<DynamoClient>,
    attributes: &HashMap<String, AttributeValue>,
    now: i64,
) -> Result<String, ServerFnError<NexusError>> {
    let check_email_not_already_exists_expression =
        format!("attribute_not_exists({})", table_attributes::EMAIL);
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
    put = put.item(
        EMAIL_VERIFICATION_REQUEST_TIME,
        AttributeValue::N(now.to_string()),
    );
    put = put.item(EMAIL_VERIFIED, AttributeValue::Bool(false));
    let put_resp = put.send().await.map_err(aws_sdk_dynamodb::Error::from);
    match put_resp {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("{:?}", e);
            Err(ServerFnError::from(match e {
                aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_) => {
                    NexusError::EmailAlreadyInUse
                }
                _ => NexusError::Unhandled,
            }))
        }
    }?;
    Ok(change_email_verification_uuid)
}

async fn send_email_for_change_email_request(
    change_email_verification_uuid: String,
    old_email: &String,
    new_email: String,
) -> Result<(), ServerFnError<NexusError>> {
    let ses_client = ses_client()?;
    let body = format!(
        "Hello,
Did you just request that your email address be changed?
If so, click on the below link to accept the email change:

https://{}/email_verification?q={}

If you did not request an email address change, please change your password.",
        SITE_FULL_DOMAIN, change_email_verification_uuid
    );
    let email_body_html = Content::builder().data(body).build().map_err(|e| {
        log::error!("Could not build email body html {:?}", e);
        UNHANDLED
    })?;
    let email_body = Body::builder().html(email_body_html).build();
    let email_subject_content = Content::builder()
        .data(format!(
            "[{}] Please verify your email address",
            SITE_DOMAIN
        ))
        .build()
        .map_err(|e| {
            log::error!("Could not build email subject content {:?}", e);
            UNHANDLED
        })?;
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
        Err(e) => Err({
            log::error!("Warning, we created a new account for {} but we weren't able to send them the verification email!", new_email);
            log::error!("{:?}", e);
            UNHANDLED
        }),
    }?;
    Ok(())
}

async fn change_value(name: &str, value: AttributeValue) -> Result<(), ServerFnError<NexusError>> {
    let client = dynamo_client()?;
    let kms_client = kms_client()?;
    let session_id = get_session_cookie().await?;
    let csrf_valid = csrf::validate_csrf_header(&(*kms_client), session_id.clone()).await?;
    if !csrf_valid {
        log::error!("Invalid CSRF");
        return Err(UNHANDLED);
    }
    let email = get_email_from_session_id(session_id, &client).await?;
    let update_resp = update_setup(&client, email)
        .update_expression("SET #e = :r")
        .expression_attribute_names("#e".to_string(), name)
        .expression_attribute_values(":r", value)
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);
    match update_resp {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }
}

pub async fn change_display_name(
    new_display_name: String,
) -> Result<(), ServerFnError<NexusError>> {
    let mut censor = Censor::from_str(&new_display_name);
    let censor_type = censor.analyze();
    if censor_type.is(Type::MODERATE_OR_HIGHER) {
        return Err(ServerFnError::from(NexusError::DisplayNameInappropriate));
    }
    let display_name_av = AttributeValue::S(new_display_name);
    change_value(table_attributes::DISPLAY_NAME, display_name_av).await
}

pub async fn change_password(new_password: String) -> Result<(), ServerFnError<NexusError>> {
    let new_password_av = AttributeValue::S(new_password);
    change_value(table_attributes::PASSWORD, new_password_av).await
}
