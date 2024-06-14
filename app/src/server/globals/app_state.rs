use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_kms::Client as KeyClient;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_ses::Client as SesClient;
use leptos::LeptosOptions;
use leptos_router::RouteListing;
use std::sync::Arc;
use stripe::Client as StripeClient;

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub dynamodb_client: Arc<DynamoClient>,
    pub ses_client: Arc<SesClient>,
    pub stripe_client: Arc<StripeClient>,
    pub s3_client: Arc<S3Client>,
    pub key_client: Arc<KeyClient>,
    pub routes: Vec<RouteListing>,
}
