use super::globals::dynamo::{query_setup, update_setup, TableKeyType};
use super::globals::dynamo::constants::*;
use super::utilities::{
    dynamo_client, extract_email_from_query, extract_email_verification_request_time_from_query,
    handle_dynamo_generic_error, ses_client,
};
use crate::{
    errors::NexusError,
    site::constants::{SITE_DOMAIN, SITE_EMAIL_ADDRESS, SITE_FULL_DOMAIN},
};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_ses::types::{Body, Content, Destination, Message};
use chrono::{DateTime, Utc};
use leptos::ServerFnError;

/// Sends an email to the given users address with a link to verify their account.
pub async fn send_verification_email(
    email_address: String,
    verification_uuid: String,
) -> Result<(), ServerFnError<NexusError>> {
    let ses_client = ses_client()?;
    let body = format!(
        "Hello,
Somebody just used this email address to sign up at {}.

If this was you, verify your email by clicking on the link below:

https://{}/email_verification/{}

If this was not you, you may ignore this email.",
        SITE_DOMAIN,
        SITE_FULL_DOMAIN,
        verification_uuid
    );
    let email_body_html = Content::builder().data(body).build().map_err(|e| {
        log::error!("Could not build email body html {:?}", e);
        NexusError::Unhandled
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
            NexusError::Unhandled
        })?;
    let configuration_set_name = format!("NexusConfigurationSet{}", std::env!("STAGE"));
    let email_message = Message::builder()
        .subject(email_subject_content)
        .body(email_body)
        .build();
    let email_send_resp = ses_client
        .send_email()
        .source(SITE_EMAIL_ADDRESS)
        .destination(Destination::builder().to_addresses(email_address).build())
        .configuration_set_name(configuration_set_name)
        .message(email_message)
        .send()
        .await
        .map_err(aws_sdk_ses::Error::from);

    match email_send_resp {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("{:?}", e);
            Err(NexusError::GenericSesError)
        }
    }?;
    Ok(())
}

/// Verifies a given email_uuid
pub async fn verify_email(email_uuid: String) -> Result<(), ServerFnError<NexusError>> {
    let client = dynamo_client()?;

    // first we have to query to find the email address associated with this verification attempt.
    let db_query_result = query_setup(&client, email_uuid, TableKeyType::EmailVerificationUUID)
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);

    let (email, email_verification_request_time) = match db_query_result {
        Ok(o) => Ok((
            extract_email_from_query(&o)?,
            extract_email_verification_request_time_from_query(&o)?,
        )),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }?;

    // TODO: Evaluate this 24 hour constant to verify email time
    let time_to_verify_email = chrono::Duration::hours(24);
    let now = Utc::now();
    let maximum_time_allowed = DateTime::from_timestamp(email_verification_request_time, 0)
        .unwrap()
        + time_to_verify_email;

    if now > maximum_time_allowed {
        return Err(ServerFnError::from(
            NexusError::EmailVerificationTookTooLong,
        ));
    }

    // secondly if we can find the email, update its verification field
    let db_update_result = update_setup(&client, email)
        .update_expression("SET #e = :r")
        .expression_attribute_names("#e".to_string(), table_attributes::EMAIL_VERIFIED)
        .expression_attribute_values(":r", AttributeValue::Bool(true))
        .send()
        .await
        .map_err(aws_sdk_dynamodb::Error::from);

    match db_update_result {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_dynamo_generic_error(e)),
    }
}
