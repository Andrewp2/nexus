use super::{
    utilities::{dynamo_client, hash_password},
    verify_email::send_verification_email,
};
use crate::{
    dynamo::constants::table_attributes::{
        self, ACCOUNT_CREATION_TIME, DISPLAY_NAME, EMAIL, EMAIL_VERIFICATION_UUID, EMAIL_VERIFIED,
        GAMES_BOUGHT, PASSWORD, USER_UUID,
    },
    env_var::get_table_name,
    errors::NexusError,
};
use aws_sdk_dynamodb::types::AttributeValue;
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
) -> Result<(), ServerFnError<NexusError>> {
    if !EmailAddress::is_valid(email.as_str()) {
        return Err(ServerFnError::from(NexusError::BadEmailAddress));
    }
    if password != password_confirmation {
        return Err(ServerFnError::from(NexusError::PasswordsNotMatching));
    }
    let mut censor = Censor::from_str(&display_name);
    let censor_type = censor.analyze();
    if censor_type.is(Type::MODERATE_OR_HIGHER) {
        return Err(ServerFnError::from(NexusError::DisplayNameInappropriate));
    }
    let dynamo_client = dynamo_client()?;
    let hashed_password = hash_password(&password).map_err(|e| {
        log::error!("Could not hash password? {:?}", e);
        ServerFnError::from(NexusError::CouldNotHashPassword)
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
        .await
        .map_err(|e| aws_sdk_dynamodb::Error::from(e));

    match db_result {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::from(match e {
            aws_sdk_dynamodb::Error::ConditionalCheckFailedException(_) => {
                NexusError::BadUsernameEmailCombination
            }
            e2 => {
                log::error!("{:?}", e2);
                NexusError::GenericDynamoServiceError
            }
        })),
    }?;

    send_verification_email(email, email_verification_uuid).await?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

