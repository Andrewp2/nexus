use leptos::{server, ServerFnError};
use uuid::Uuid;

use crate::{
    auth::utilities::session_lifespan,
    dynamo::constants::{
        index::SESSION_ID,
        table_attributes::{EMAIL, EMAIL_VERIFIED, PASSWORD, SESSION_EXPIRY},
    },
    errors::NexusServerError,
    site::constants::{SITE_DOMAIN, SITE_EMAIL_ADDRESS},
};

#[cfg(feature = "ssr")]
pub mod utilities {
    use argon2::{
        password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };
    use aws_sdk_dynamodb::Client as DynamoClient;
    use aws_sdk_ses::Client as SesClient;
    use leptos::{use_context, ServerFnError};
    use rand::rngs::OsRng;

    pub fn dynamo_client() -> Result<DynamoClient, ServerFnError> {
        use_context::<DynamoClient>()
            .ok_or_else(|| ServerFnError::ServerError("Dynamo client missing.".into()))
    }

    pub fn ses_client() -> Result<SesClient, ServerFnError> {
        use_context::<SesClient>()
            .ok_or_else(|| ServerFnError::ServerError("Ses client missing".into()))
    }

    pub fn hash_password(password: &str) -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }

    pub fn verify_password(password: &str, database_hash: &str) -> bool {
        if let Ok(hash) = PasswordHash::new(&database_hash) {
            return Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_ok();
        }
        false
    }

    pub fn session_lifespan(remember: bool) -> chrono::Duration {
        match remember {
            true => chrono::Duration::hours(3),
            false => chrono::Duration::days(60),
        }
    }
}

/// Sends an email to the given users address with a link to verify their account.
async fn send_verification_email(
    email_address: String,
    verification_uuid: String,
) -> Result<(), ServerFnError> {
    use self::utilities::ses_client;

    use aws_sdk_ses::{
        operation::send_email::SendEmailError,
        types::{Body, Content, Destination, Message},
    };

    let ses_client = ses_client()?;

    let body = format!(
        "Hello,
    Somebody just used this email address to sign up at Mysite.
    
    If this was you, verify your email by clicking on the link below:
    
    https://MySite.com/email-verification/{}
    
    If this was not you, you may ignore this email.",
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
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                SendEmailError::AccountSendingPausedException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericSesError
                }
                SendEmailError::ConfigurationSetDoesNotExistException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericSesError
                }
                SendEmailError::ConfigurationSetSendingPausedException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericSesError
                }
                SendEmailError::MailFromDomainNotVerifiedException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericSesError
                }
                SendEmailError::MessageRejected(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericSesError
                }
                _ => {
                    log::error!("Catchall SES error in send_email function");
                    NexusServerError::GenericSesError
                }
            }
            .to_string(),
        )),
    }?;

    Ok(())
}

/// Server function that signs the user up.
/// Sends an email to the given email address.
#[server(Signup, "/api")]
pub async fn signup(
    display_name: String,
    email: String,
    password: String,
    password_confirmation: String,
) -> Result<(), ServerFnError> {
    use self::utilities::{dynamo_client, hash_password};
    use crate::dynamo::constants::*;
    use aws_sdk_dynamodb::{operation::put_item::PutItemError, types::AttributeValue};
    use chrono::Utc;
    use rustrict::{Censor, Type};
    use uuid::Uuid;

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "Passwords did not match.".to_string(),
        ));
    }
    let mut censor = Censor::from_str(&display_name);
    let censor_type = censor.analyze();
    if censor_type.is(Type::MODERATE_OR_HIGHER) {
        return Err(ServerFnError::ServerError(
            "Display name is bad".to_string(),
        ));
    }
    let dynamo_client = dynamo_client()?;
    let hashed_password = hash_password(&password)
        .map_err(|_| ServerFnError::ServerError("Could not hash password?".into()))?;
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
        .table_name(TABLE_NAME)
        .item(table_attributes::DISPLAY_NAME, display_name_av)
        .item(table_attributes::EMAIL, email_av)
        .item(table_attributes::PASSWORD, hashed_password_av)
        .item(table_attributes::GAMES_BOUGHT, games_bought_av)
        .item(table_attributes::USER_UUID, uuid_av)
        .item(
            table_attributes::EMAIL_VERIFIED,
            AttributeValue::Bool(false),
        )
        .item(
            table_attributes::EMAIL_VERIFICATION_UUID,
            AttributeValue::S(email_verification_uuid.clone()),
        )
        .item(
            table_attributes::ACCOUNT_CREATION_TIME,
            AttributeValue::N(Utc::now().timestamp().to_string()),
        )
        .condition_expression(check_email_not_exists_expression)
        .send()
        .await;

    match db_result {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                // TODO: Think about error messages
                PutItemError::ConditionalCheckFailedException(_) => {
                    NexusServerError::BadUsernameEmailCombination
                }
                PutItemError::InternalServerError(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                PutItemError::InvalidEndpointException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                PutItemError::ItemCollectionSizeLimitExceededException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                PutItemError::ProvisionedThroughputExceededException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                PutItemError::RequestLimitExceeded(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                PutItemError::ResourceNotFoundException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                PutItemError::TransactionConflictException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                _ => {
                    log::error!("Catchall case in signup PutItemError");
                    NexusServerError::GenericDynamoServiceError
                }
            }
            .to_string(),
        )),
    }?;
    send_verification_email(email, email_verification_uuid).await?;
    leptos_axum::redirect("/email_verification/");
    Ok(())
}

/// Logs the given user in
#[server(Login, "/api")]
pub async fn login(email: String, password: String, remember: bool) -> Result<(), ServerFnError> {
    use self::utilities::{dynamo_client, verify_password};
    use crate::dynamo::constants::*;
    use crate::errors::NexusServerError;
    use aws_sdk_dynamodb::{
        operation::{query::QueryError, update_item::UpdateItemError},
        types::AttributeValue,
    };
    use chrono::Utc;

    // TODO: Implement rate limiting? Handle on API Gateway instead of here maybe?

    let client = dynamo_client()?;
    let columns_to_query = vec![EMAIL, PASSWORD, EMAIL_VERIFIED];
    let check_if_password_exists_filter_expression = format!("attribute_exists({})", PASSWORD);
    let key_condition = format!("{} = :email_val", EMAIL);
    let db_result = client
        .query()
        .limit(1)
        .table_name(TABLE_NAME)
        .key_condition_expression(key_condition)
        .expression_attribute_values(":email_val", AttributeValue::S(email.clone()))
        .projection_expression(columns_to_query.join(", "))
        .filter_expression(check_if_password_exists_filter_expression)
        .send()
        .await;

    let password_database_hash = match db_result {
        Ok(val) => {
            let item = val.items().first().ok_or(ServerFnError::ServerError(
                NexusServerError::CouldNotFindRowWithThatEmail.to_string(),
            ))?;
            let blob = item
                .get(PASSWORD)
                .ok_or_else(|| -> ServerFnError {
                    log::error!("Was not able to find the password, despite the filter expression");
                    ServerFnError::ServerError(
                        NexusServerError::GenericDynamoServiceError.to_string(),
                    )
                })?
                .as_b()
                .map_err(|e| -> ServerFnError {
                    log::error!(
                        "Was not able to get the inner blob from the password {:?}",
                        e
                    );
                    ServerFnError::ServerError(NexusServerError::Unhandled.to_string())
                })?;
            let hash_string =
                String::from_utf8(blob.clone().into_inner()).map_err(|e| -> ServerFnError {
                    log::error!(
                        "Was not able to get a utf8 string from the blob {:?}, {:?}",
                        blob,
                        e
                    );
                    ServerFnError::ServerError(NexusServerError::Unhandled.to_string())
                })?;
            Ok(hash_string)
        }
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                QueryError::InternalServerError(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::InvalidEndpointException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::ProvisionedThroughputExceededException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::RequestLimitExceeded(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::ResourceNotFoundException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                _ => {
                    log::error!("Catchall case for query in login");
                    NexusServerError::GenericDynamoServiceError
                }
            }
            .to_string(),
        )),
    }?;

    match verify_password(&password, &password_database_hash) {
        true => {
            let lifespan = session_lifespan(remember);
            let now = Utc::now();
            let future_time = now + lifespan;
            let lifespan_av = AttributeValue::N(future_time.timestamp().to_string());
            let email_av = AttributeValue::S(email);
            let session_uuid = Uuid::new_v4().to_string();
            let session_uuid_av = AttributeValue::S(session_uuid);
            let update_expression = format!(
                "SET {} = :session_id, {} = :session_expiry",
                SESSION_ID, SESSION_EXPIRY
            );
            let db_result = client
                .update_item()
                .table_name(TABLE_NAME)
                .key(EMAIL, email_av)
                .update_expression(update_expression)
                .expression_attribute_values(":session_id", session_uuid_av)
                .expression_attribute_values(":session_expiry", lifespan_av)
                .send()
                .await;
            match db_result {
                Ok(_) => Ok(()),
                Err(e) => Err(ServerFnError::ServerError(
                    match e.into_service_error() {
                        UpdateItemError::ConditionalCheckFailedException(_) => {
                            NexusServerError::EmailNotFoundLogin
                        }
                        UpdateItemError::InternalServerError(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        UpdateItemError::InvalidEndpointException(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        UpdateItemError::ItemCollectionSizeLimitExceededException(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        UpdateItemError::ProvisionedThroughputExceededException(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        UpdateItemError::RequestLimitExceeded(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        UpdateItemError::ResourceNotFoundException(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        UpdateItemError::TransactionConflictException(e) => {
                            log::error!("{:?}", e);
                            NexusServerError::GenericDynamoServiceError
                        }
                        _ => {
                            log::error!("Catchall case in updating session_id after logging in");
                            NexusServerError::GenericDynamoServiceError
                        }
                    }
                    .to_string(),
                )),
            }
        }
        false => Err(ServerFnError::ServerError(
            "Password does not match.".to_string(),
        )),
    }
}

#[server(VerifyEmail, "/api")]
pub async fn verify_email(email_uuid: String) -> Result<(), ServerFnError> {
    use self::utilities::dynamo_client;
    use crate::dynamo::constants::*;
    use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
    use aws_sdk_dynamodb::{operation::query::QueryError, types::AttributeValue};

    let client = dynamo_client()?;
    // TODO: Fix shit

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
        Ok(o) => {
            let items = o.items.ok_or(ServerFnError::ServerError(
                NexusServerError::EmailVerificationUuidNotFound.to_string(),
            ))?;
            let item = items.first().ok_or(ServerFnError::ServerError(
                NexusServerError::EmailVerificationUuidNotFound.to_string(),
            ))?;
            let email_string = item
                .get(table_attributes::EMAIL)
                .ok_or(ServerFnError::ServerError(
                    NexusServerError::Unhandled.to_string(),
                ))?
                .as_s()
                .map_err(|e| {
                    log::error!("Could not get email as string {:?}", e);
                    ServerFnError::ServerError(NexusServerError::Unhandled.to_string())
                })?;

            Ok(email_string)
        }
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                QueryError::InternalServerError(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::InvalidEndpointException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::ProvisionedThroughputExceededException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::RequestLimitExceeded(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                QueryError::ResourceNotFoundException(e) => {
                    log::error!("{:?}", e);
                    NexusServerError::GenericDynamoServiceError
                }
                _ => {
                    log::error!("Catchall in verify_email query");
                    NexusServerError::GenericDynamoServiceError
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
        Err(e) => Err(ServerFnError::ServerError(
            match e.into_service_error() {
                UpdateItemError::ConditionalCheckFailedException(_) => todo!(),
                UpdateItemError::InternalServerError(_) => todo!(),
                UpdateItemError::InvalidEndpointException(_) => todo!(),
                UpdateItemError::ItemCollectionSizeLimitExceededException(_) => todo!(),
                UpdateItemError::ProvisionedThroughputExceededException(_) => todo!(),
                UpdateItemError::RequestLimitExceeded(_) => todo!(),
                UpdateItemError::ResourceNotFoundException(_) => todo!(),
                UpdateItemError::TransactionConflictException(_) => todo!(),
                _ => todo!(),
            }
            .to_string(),
        )),
    }
}

