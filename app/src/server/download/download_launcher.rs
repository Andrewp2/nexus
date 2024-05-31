use axum::{
    body::Body,
    extract::{Extension, Path},
    response::IntoResponse,
};
use http::{Response, StatusCode};

use super::super::globals::app_state::AppState;

use super::download_utils::{download_file_from_s3, SessionId, LAUNCHER_BUCKET_NAME};

pub async fn download_launcher(
    Path(os_type): Path<String>,
    Extension(state): Extension<AppState>,
    _session_id: SessionId,
) -> impl IntoResponse {
    // TODO: Check content_types
    let (launcher_key, content_type) = match os_type.as_str() {
        "windows" => (
            "launcher.msi",
            "application/vnd.microsoft.portable-executable",
        ),
        "macos" => ("launcher.dmg", "application/x-apple-diskimage"),
        "linux" => ("launcher.AppImage", "application/octet-stream"),
        _ => return (StatusCode::BAD_REQUEST, "Unsupported platform").into_response(),
    };

    match download_file_from_s3(
        &state.s3_client,
        LAUNCHER_BUCKET_NAME.to_owned(),
        launcher_key.to_owned(),
    )
    .await
    {
        Ok(file_bytes) => {
            let file_name = launcher_key
                .split('/')
                .last()
                .expect("Invalid launcher file path");
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", file_name),
                )
                .body(Body::from(file_bytes))
                .unwrap()
        }
        Err(error) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(error))
            .unwrap(),
    }
}
