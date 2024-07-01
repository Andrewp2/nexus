use super::utilities::{
    dynamo_client, handle_dynamo_generic_error, session_lifespan, verify_password,
};
use super::{
    globals::{
        dynamo::constants::table_attributes::{
            EMAIL, EMAIL_VERIFIED, PASSWORD, SESSION_EXPIRY, SESSION_ID,
        },
        env_var::{get_host_prefix, get_table_name},
    },
    utilities::kms_client,
};
use crate::errors::NexusError;
use async_trait::async_trait;
use aws_sdk_dynamodb::{operation::query::QueryOutput, types::AttributeValue};
use aws_sdk_kms::operation::generate_mac::GenerateMacOutput;
use aws_sdk_kms::operation::verify_mac::VerifyMacOutput;
use aws_sdk_kms::types::MacAlgorithmSpec;
use aws_sdk_kms::{primitives::Blob, Client as KeyClient};
use base64::prelude::*;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use http::{header, HeaderMap, HeaderValue};
use leptos::{expect_context, ServerFnError};
use leptos_axum::{extract, ResponseOptions};
use rand::rngs::OsRng;
use rand::Rng;
use uuid::Uuid;

/// Generates a cryptographically random vec of bytes
pub fn generate_random_bytes() -> Vec<u8> {
    let mut rng = OsRng;
    let mut bytes = vec![0u8; 16];
    rng.fill(&mut bytes[..]);
    bytes
}

#[async_trait]
pub trait KmsClientTrait {
    async fn generate_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
        algorithm: MacAlgorithmSpec,
    ) -> Result<GenerateMacOutput, ServerFnError<NexusError>>;

    async fn verify_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
        algorithm: MacAlgorithmSpec,
        mac: Blob,
    ) -> Result<VerifyMacOutput, ServerFnError<NexusError>>;
}

#[async_trait]
impl KmsClientTrait for KeyClient {
    async fn generate_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
        algorithm: MacAlgorithmSpec,
    ) -> Result<GenerateMacOutput, ServerFnError<NexusError>> {
        self.generate_mac()
            .key_id(key_id)
            .message(Blob::new(message))
            .mac_algorithm(algorithm)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to generate MAC: {:?}", e);
                ServerFnError::from(NexusError::Unhandled)
            })
    }

    async fn verify_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
        algorithm: MacAlgorithmSpec,
        mac: Blob,
    ) -> Result<VerifyMacOutput, ServerFnError<NexusError>> {
        self.verify_mac()
            .key_id(key_id)
            .message(Blob::new(message))
            .mac_algorithm(algorithm)
            .mac(mac)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to generate MAC: {:?}", e);
                ServerFnError::from(NexusError::Unhandled)
            })
    }
}

pub async fn generate_csrf_token<T: KmsClientTrait>(
    kms_client: &T,
    session_id: String,
    random_bytes: Vec<u8>,
) -> Result<String, ServerFnError<NexusError>> {
    let key_id = format!("alias/CSRFSecretKey{}", std::env!("STAGE"));
    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
    assert!(!session_id.contains("!"));
    assert!(!random_string.contains("!"));
    let message = format!("{}!{}", session_id, random_string);
    let mac_result = kms_client
        .generate_mac(
            key_id,
            message.clone().into_bytes(),
            MacAlgorithmSpec::HmacSha256,
        )
        .await?;
    let mac = mac_result.mac.unwrap();
    let mac_string = general_purpose::URL_SAFE_NO_PAD.encode(mac);
    assert!(!mac_string.contains("."));
    assert!(!message.contains("."));
    let csrf_token = format!("{}.{}", mac_string, message);
    Ok(csrf_token)
}

pub async fn validate_csrf_token<T: KmsClientTrait>(
    kms_client: &T,
    session_id: String,
) -> Result<bool, ServerFnError<NexusError>> {
    let key_id = format!("alias/CSRFSecretKey{}", std::env!("STAGE"));
    let headers: HeaderMap = extract().await.map_err(|e| {
        log::error!("{:?}", e);
        ServerFnError::from(NexusError::Unhandled)
    })?;
    let csrf_header = headers.get("X-Csrf-Token").unwrap().to_str().unwrap();
    let mut parts = csrf_header.splitn(2, ".");
    let csrf_token = parts.next().unwrap_or("");
    let message = parts.next().unwrap_or("");
    let verification = kms_client
        .verify_mac(
            key_id,
            BASE64_STANDARD_NO_PAD.decode(message).unwrap(),
            MacAlgorithmSpec::HmacSha256,
            Blob::new(csrf_token.as_bytes()),
        )
        .await?;
    Ok(verification.mac_valid())
}
