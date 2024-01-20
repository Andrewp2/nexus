use super::{
    utilities::{check_email_uniqueness, dynamo_client, hash_password},
    verify_email::send_verification_email,
};
use crate::{
    dynamo::constants::{
        get_table_name,
        table_attributes::{
            self, ACCOUNT_CREATION_TIME, DISPLAY_NAME, EMAIL, EMAIL_VERIFICATION_UUID,
            EMAIL_VERIFIED, GAMES_BOUGHT, PASSWORD, USER_UUID,
        },
    },
    errors::NexusError,
};
use aws_sdk_dynamodb::{error::SdkError, operation::put_item::PutItemError, types::AttributeValue};
use chrono::Utc;
use email_address::EmailAddress;
use leptos::ServerFnError;
use rustrict::{Censor, Type};
use uuid::Uuid;

pub async fn signup(
    display_name: String,
    email: String,
    password: String,
    password_confirmation: String,
) -> Result<(), ServerFnError> {
    if !EmailAddress::is_valid(email.as_str()) {
        return Err(ServerFnError::ServerError(
            NexusError::BadEmailAddress.to_string(),
        ));
    }
    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            NexusError::PasswordsNotMatching.to_string(),
        ));
    }
    let mut censor = Censor::from_str(&display_name);
    let censor_type = censor.analyze();
    if censor_type.is(Type::MODERATE_OR_HIGHER) {
        return Err(ServerFnError::ServerError(
            NexusError::DisplayNameInappropriate.to_string(),
        ));
    }
    let dynamo_client = dynamo_client()?;
    let hashed_password = hash_password(&password).map_err(|e| {
        log::error!("Could not hash password? {:?}", e);
        ServerFnError::ServerError(NexusError::CouldNotHashPassword.to_string())
    })?;
    let display_name_av = AttributeValue::S(display_name);
    let email_av = AttributeValue::S(email.clone());
    let games_bought_av = AttributeValue::L(Vec::new());
    let hashed_password_av = AttributeValue::B(aws_sdk_dynamodb::primitives::Blob::new(
        hashed_password.to_string(),
    ));
    let uuid = Uuid::new_v4().to_string();
    let uuid_av = AttributeValue::S(uuid);
    let email_verification_uuid = Uuid::new_v4().to_string();
    let check_email_not_exists_expression =
        format!("attribute_not_exists({})", table_attributes::EMAIL);
    let db_result = dynamo_client
        .put_item()
        .table_name(get_table_name())
        .item(DISPLAY_NAME, display_name_av)
        .item(EMAIL, email_av)
        .item(PASSWORD, hashed_password_av)
        .item(GAMES_BOUGHT, games_bought_av)
        .item(USER_UUID, uuid_av)
        .item(EMAIL_VERIFIED, AttributeValue::Bool(false))
        .item(
            EMAIL_VERIFICATION_UUID,
            AttributeValue::S(email_verification_uuid.clone()),
        )
        .item(
            ACCOUNT_CREATION_TIME,
            AttributeValue::N(Utc::now().timestamp().to_string()),
        )
        .condition_expression(check_email_not_exists_expression)
        .send()
        .await;

    match db_result {
        Ok(_) => Ok(()),
        Err(e) => Err(handle_signup_put_error(e)),
    }?;
    send_verification_email(email, email_verification_uuid).await?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

fn handle_signup_put_error(e: SdkError<PutItemError>) -> ServerFnError {
    ServerFnError::ServerError(
        match e.into_service_error() {
            PutItemError::ConditionalCheckFailedException(_) => {
                NexusError::BadUsernameEmailCombination
            }
            PutItemError::InternalServerError(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            PutItemError::InvalidEndpointException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            PutItemError::ItemCollectionSizeLimitExceededException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            PutItemError::ProvisionedThroughputExceededException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            PutItemError::RequestLimitExceeded(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            PutItemError::ResourceNotFoundException(e) => {
                log::error!("{:?}", e);
                NexusError::GenericDynamoServiceError
            }
            PutItemError::TransactionConflictException(e) => {
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

