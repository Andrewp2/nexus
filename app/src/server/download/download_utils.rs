use super::super::globals::env_var::get_host_prefix;
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Request},
};
use http::{HeaderName, StatusCode};

pub const GAME_BUCKET_NAME: &str = "games";
pub const LAUNCHER_BUCKET_NAME: &str = "launchers";

pub struct SessionId {
    pub session_id: String,
}

#[async_trait]
impl<S: Sync> FromRequest<S, Body> for SessionId {
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        // Define the custom header name
        let header = format!("{}session-id", get_host_prefix());
        let header_name = HeaderName::from_lowercase(header.as_bytes()).unwrap();

        // Attempt to extract the session-id header
        match req.headers().get(&header_name) {
            Some(header_value) => {
                // Attempt to convert the header value to a string
                match header_value.to_str() {
                    Ok(value) => Ok(SessionId {
                        session_id: value.to_string(),
                    }),
                    Err(_) => Err((StatusCode::BAD_REQUEST, "Invalid session-id header value")),
                }
            }
            None => Err((
                StatusCode::UNAUTHORIZED,
                "Unauthorized: Missing session-id header",
            )),
        }
    }
}

pub async fn download_file_from_s3(
    s3_client: &S3Client,
    bucket: String,
    key: String,
) -> Result<Bytes, String> {
    let resp = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data = resp.body.collect().await.map_err(|e| e.to_string())?;

    Ok(data.into_bytes())
}

// pub async fn download() -> Result<String, ServerFnError<NexusError>> {
//     let client = dynamo_client()?;
//     let session_id = get_session_cookie().await?;
//     let db_query_result = query_setup(&client, session_id, TableKeyType::SessionId)
//         .send()
//         .await
//         .map_err(|e| aws_sdk_dynamodb::Error::from(e));

//     let email = match db_query_result {
//         Ok(o) => Ok(extract_id_from_query(o)?),
//         Err(e) => Err(handle_dynamo_generic_error(e)),
//     }?;

//     Ok(email)
// }
