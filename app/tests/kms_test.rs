use app::server::csrf::{generate_csrf_token, generate_random_bytes, KmsClientTrait};
use aws_sdk_kms::{
    operation::{generate_mac::GenerateMacOutput, verify_mac::VerifyMacOutput},
    primitives::Blob,
    types::MacAlgorithmSpec,
};
use base64::{engine::general_purpose, Engine as _};
use leptos::ServerFnError;
use mockall::{mock, predicate::*};
use uuid::Uuid;

mock! {
    KeyClient {}
    #[async_trait::async_trait]
    impl KmsClientTrait for KeyClient {
        async fn generate_mac(
            &self,
            key_id: String,
            message: Vec<u8>,
            algorithm: MacAlgorithmSpec,
        ) -> Result<GenerateMacOutput, ServerFnError<app::errors::NexusError>>;

        async fn verify_mac(
            &self,
            key_id: String,
            message: Vec<u8>,
            algorithm: MacAlgorithmSpec,
            mac: Blob,
        ) -> Result<VerifyMacOutput, ServerFnError<app::errors::NexusError>>;
    }
}

#[tokio::test]
async fn test_generate_csrf_token_success() {
    let mut mock_client = MockKeyClient::new();
    let session_id = "test_session_id".to_string();
    let expected_mac = vec![1, 2, 3, 4];
    let e_mac = expected_mac.clone();
    let s_id = session_id.clone();
    mock_client
        .expect_generate_mac()
        .withf(move |key_id, message, algorithm| {
            key_id == &format!("alias/CSRFSecretKey{}", std::env!("STAGE"))
                && algorithm == &MacAlgorithmSpec::HmacSha256
                && message.starts_with(s_id.as_bytes())
        })
        .returning(move |_, _, _| {
            Ok(GenerateMacOutput::builder()
                .mac(Blob::new(e_mac.clone()))
                .build())
        });
    let random_bytes = generate_random_bytes();
    let result = generate_csrf_token(&mock_client, session_id.clone(), random_bytes).await;
    assert!(result.is_ok());
    let csrf_token = result.unwrap();
    let parts: Vec<&str> = csrf_token.split('.').collect();
    assert_eq!(parts.len(), 2);
    let mac_part = general_purpose::URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
    assert_eq!(mac_part, expected_mac);
    let message_parts: Vec<&str> = parts[1].split('!').collect();
    assert_eq!(message_parts.len(), 2);
    assert_eq!(message_parts[0], session_id);
}

#[tokio::test]
async fn assert_uuid_clean() {
    for _ in 0..10000 {
        let session_id = Uuid::new_v4().to_string();
        let random_bytes = generate_random_bytes();
        let random_string = general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
        assert!(!session_id.contains("!"));
        assert!(!random_string.contains("!"));
        assert!(!session_id.contains("."));
        assert!(!random_string.contains("."));
    }
}
