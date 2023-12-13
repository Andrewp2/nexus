use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_ses::Client as SesClient;
use axum::extract::FromRef;
use leptos::LeptosOptions;
use leptos_router::RouteListing;
use std::sync::Arc;

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub dynamodb_client: Arc<DynamoClient>,
    pub ses_client: Arc<SesClient>,
    pub routes: Vec<RouteListing>,
}

