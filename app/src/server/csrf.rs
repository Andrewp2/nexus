use crate::errors::{NexusError, UNHANDLED};
use async_trait::async_trait;
use aws_sdk_kms::{
    operation::{generate_mac::GenerateMacOutput, verify_mac::VerifyMacOutput},
    types::MacAlgorithmSpec,
};
use aws_sdk_kms::{primitives::Blob, Client as KeyClient};
use base64::{engine::general_purpose, prelude::*};
use http::HeaderMap;
use leptos::ServerFnError;
use leptos_axum::extract;
use rand::{rngs::OsRng, Rng};

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
    ) -> Result<GenerateMacOutput, ServerFnError<NexusError>>;

    async fn verify_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
        mac: Blob,
    ) -> Result<VerifyMacOutput, ServerFnError<NexusError>>;
}

#[async_trait]
impl KmsClientTrait for KeyClient {
    async fn generate_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
    ) -> Result<GenerateMacOutput, ServerFnError<NexusError>> {
        self.generate_mac()
            .key_id(key_id)
            .message(Blob::new(message))
            .mac_algorithm(MacAlgorithmSpec::HmacSha256)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to generate MAC: {:?}", e);
                UNHANDLED
            })
    }

    async fn verify_mac(
        &self,
        key_id: String,
        message: Vec<u8>,
        mac: Blob,
    ) -> Result<VerifyMacOutput, ServerFnError<NexusError>> {
        self.verify_mac()
            .key_id(key_id)
            .message(Blob::new(message))
            .mac_algorithm(MacAlgorithmSpec::HmacSha256)
            .mac(mac)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to generate MAC: {:?}", e);
                UNHANDLED
            })
    }
}

// Generates a new CSRF token
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
        .generate_mac(key_id, message.clone().into_bytes())
        .await?;
    let mac = mac_result.mac.unwrap();
    let mac_string = general_purpose::URL_SAFE_NO_PAD.encode(mac);
    assert!(!mac_string.contains("."));
    assert!(!message.contains("."));
    let csrf_token = format!("{}.{}", mac_string, message);
    Ok(csrf_token)
}

/// Validates the CSRF header
pub async fn validate_csrf_header<T: KmsClientTrait>(
    kms_client: &T,
    session_id: String,
) -> Result<bool, ServerFnError<NexusError>> {
    let key_id = format!("alias/CSRFSecretKey{}", std::env!("STAGE"));
    let headers: HeaderMap = extract().await.map_err(|e| {
        log::error!("Can't get headers from request{:?}", e);
        UNHANDLED
    })?;
    let csrf_header = headers
        .get("X-Csrf-Token")
        .ok_or_else(|| UNHANDLED)?
        .to_str()
        .map_err(|_| UNHANDLED)?;
    let mut csrf_header_parts = csrf_header.splitn(2, ".");
    let csrf_token = csrf_header_parts.next().ok_or_else(|| UNHANDLED)?;
    let message = csrf_header_parts.next().ok_or_else(|| UNHANDLED)?;
    // We need to verify that the CSRF token that came from the header
    // is the same one as that came with this particular session_id
    // Otherwise an attacker could just use their own CSRF token
    // (although they would have to find a way to use their own CSRF token on
    // someone else's browser... XSS attack maybe?)
    let mut message_parts = message.splitn(2, "!");
    let session_id_in_header = message_parts.next().ok_or_else(|| UNHANDLED)?;
    use subtle::ConstantTimeEq;
    // We might be calling verify_csrf_token before we've verified that the session is legitimate
    // In that case, attackers could use a timing side channel attack to figure out the CSRF token value
    // byte by byte. I do not think this actually is insecure as you would need to perform the CSRF attack many
    // times over, and it's hard to get a user to click a CSRF attack link thousands of times, however it's
    // better to be safe than sorry.
    let equal: bool = session_id
        .as_bytes()
        .ct_eq(session_id_in_header.as_bytes())
        .into();
    if !equal {
        return Err(UNHANDLED);
    }
    let verification = kms_client
        .verify_mac(
            key_id,
            BASE64_URL_SAFE_NO_PAD
                .decode(message)
                .map_err(|_| UNHANDLED)?,
            Blob::new(csrf_token.as_bytes()),
        )
        .await?;
    Ok(verification.mac_valid())
}
