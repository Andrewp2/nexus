use super::globals::{
    self,
    dynamo::{
        self,
        constants::{
            index::EMAIL_VERIFICATION_UUID,
            table_attributes::{self, EMAIL, EMAIL_VERIFIED, SESSION_EXPIRY, SESSION_ID},
        },
        query_builder, query_entire_user, query_setup,
    },
    env_var::get_table_name,
};
use super::utilities::{
    dynamo_client, get_email_from_session_id, get_session_cookie, handle_dynamo_generic_error,
    ses_client,
};
use crate::errors::NexusError;
use crate::site::constants::{SITE_DOMAIN, SITE_EMAIL_ADDRESS, SITE_FULL_DOMAIN};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_ses::types::{Body, Content, Destination, Message};
use chrono::Utc;
use email_address::EmailAddress;
use leptos::ServerFnError;
use rustrict::{Censor, Type};
use uuid::Uuid;

/// Starts a request to change email
pub async fn change_email_request(new_email: String) -> Result<(), ServerFnError<NexusError>> {
    if !EmailAddress::is_valid(&new_email) {
        return Err(ServerFnError::from(NexusError::BadEmailAddress));
    }
    let session_id_cookie = get_session_cookie().await?;
    let client = dynamo_client()?;
    let old_user_query = query_setup(
        &client,
        session_id_cookie.clone(),
        dynamo::TableKeyType::SessionId,
    )
    .send()
    .await
    .map_err(aws_sdk_dynamodb::Error::from);
    let user = match old_user_query {
        Ok(o) => Ok(o),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }?;
    let items = user.items.ok_or_else(|| {
        log::error!("Could not get items from user");
        ServerFnError::from(NexusError::Unhandled)
    })?;
    let attributes = items.first().ok_or_else(|| {
        log::error!("Could not get first item from items");
        ServerFnError::from(NexusError::Unhandled)
    })?;
    let session_expiry = attributes
        .get(SESSION_EXPIRY)
        .ok_or_else(|| {
            log::error!("Could not get session expiry");
            ServerFnError::from(NexusError::Unhandled)
        })?
        .as_n()
        .map_err(|e| {
            log::error!("Could not convert session expiry to number {:?}", e);
            ServerFnError::from(NexusError::Unhandled)
        })?
        .parse::<i64>()
        .map_err(|e| {
            log::error!("Could not parse sesion expiry number as i64 {:?}", e);
            ServerFnError::from(NexusError::Unhandled)
        })?;
    let now = Utc::now().timestamp();
    if now >= session_expiry {
        return Err(ServerFnError::from(NexusError::InvalidSession));
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
    // let old_email = old_user_query.email.unwrap();
    let old_email = attributes
        .get(EMAIL)
        .ok_or_else(|| {
            log::error!("Could not get email attribute");
            ServerFnError::from(NexusError::Unhandled)
        })?
        .as_s()
        .map_err(|e| {
            log::error!("Could not get email attribute as string {:?}", e);
            ServerFnError::from(NexusError::Unhandled)
        })?;
    let ses_client = ses_client()?;
    let body = format!(
        "Hello,
Did you just request that your email address be changed?
If so, click on the below link to accept the email change:

https://{}/email_verification/{}

If you did not request an email address change, please change your password.",
        SITE_FULL_DOMAIN,
        change_email_verification_uuid.to_string()
    );
    let email_body_html = Content::builder().data(body).build().map_err(|e| {
        log::error!("Could not build email body html {:?}", e);
        ServerFnError::from(NexusError::Unhandled)
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
            ServerFnError::from(NexusError::Unhandled)
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
            ServerFnError::from(NexusError::Unhandled)
        }),
    }?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

async fn change_value(name: &str, value: AttributeValue) -> Result<(), ServerFnError<NexusError>> {
    let client = dynamo_client()?;
    let session_id = get_session_cookie().await?;
    let email = get_email_from_session_id(session_id, &client).await?;
    let update_resp = client
        .update_item()
        .table_name(get_table_name())
        .key(table_attributes::EMAIL, AttributeValue::S(email))
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

// TODO: Fix
pub async fn change_password(new_password: String) -> Result<(), ServerFnError<NexusError>> {
    let new_password_av = AttributeValue::S(new_password);
    change_value(table_attributes::PASSWORD, new_password_av).await
}
