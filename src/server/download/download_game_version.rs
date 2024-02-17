use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::Client as S3Client;
use axum::{
    body::Body,
    extract::{Extension, Path},
    response::{IntoResponse, Response as HttpResponse},
};
use http::StatusCode;
use semver::Version;

use crate::{
    app_state::AppState,
    dynamo::constants::{index, table_attributes},
    env_var::get_table_name,
    server::utilities::check_if_session_is_valid,
};

use super::download_utils::{download_file_from_s3, SessionId, GAME_BUCKET_NAME};

fn handle_error(msg: String) -> HttpResponse {
    log::error!("{}", msg);
    return (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response();
}

pub async fn download_game_version(
    Path((game, platform, version)): Path<(String, String, String)>,
    Extension(state): Extension<AppState>,
    session_id: SessionId,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // TODO: Authentication
    let dynamo_client = state.dynamodb_client;
    let session_id = session_id.session_id;
    let session_valid_result = check_if_session_is_valid(session_id.clone(), &dynamo_client).await;

    let unhandled_error = || -> HttpResponse { handle_error("unhandled".to_string()) };

    if session_valid_result.is_err() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Session expired or otherwise invalid",
        )
            .into_response());
    }

    let (valid, email) = session_valid_result.unwrap();

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Session expired or otherwise invalid",
        )
            .into_response());
    }

    // TODO: We are authenticated, but we need to check if we can download the game version
    let db_query_result = dynamo_client
        .query()
        .limit(1)
        .table_name(get_table_name())
        .key_condition_expression("#k = :v")
        .expression_attribute_names("k".to_string(), table_attributes::EMAIL)
        .expression_attribute_values(":v".to_string(), AttributeValue::S(email))
        .send()
        .await
        .map_err(|_| unhandled_error())?;

    let bought_game = db_query_result
        .items
        .ok_or_else(|| unhandled_error())?
        .first()
        .ok_or_else(|| unhandled_error())?
        .get(table_attributes::GAMES_BOUGHT)
        .ok_or_else(|| unhandled_error())?
        .as_l()
        .map_err(|_| unhandled_error())?
        .iter()
        .map(|i| i.as_s().unwrap_or(&("".to_string())).to_string())
        .collect::<Vec<String>>()
        .contains(&game);

    if !bought_game {
        return Err((StatusCode::UNAUTHORIZED, "Hasn't bought game").into_response());
    }

    let version_path = if version == "latest" {
        match find_latest_version(&state.s3_client, &platform, &game).await {
            Ok(v) => v,
            Err(error) => return Err((StatusCode::INTERNAL_SERVER_ERROR, error).into_response()),
        }
    } else {
        format!("{}/{}/{}/game.zip", game, platform, version) // Adjust this format as necessary
    };

    match download_file_from_s3(&state.s3_client, GAME_BUCKET_NAME.to_owned(), version_path).await {
        Ok(file_bytes) => Ok(HttpResponse::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/zip")
            .header(
                "Content-Disposition",
                format!(
                    "attachment; filename=\"{}-{}-{}.zip\"",
                    game, platform, version
                ),
            )
            .body(Body::from(file_bytes))
            .unwrap()),
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, error).into_response()),
    }
}

pub async fn find_latest_version(
    s3_client: &S3Client,
    platform: &str,
    game: &str,
) -> Result<String, String> {
    let prefix = format!("{}/{}/", game, platform);
    let resp = s3_client
        .list_objects_v2()
        .bucket(GAME_BUCKET_NAME)
        .prefix(&prefix)
        .delimiter("/") // Important to treat the version folders as distinct entities
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // TODO: this code seems suspicious
    let versions = resp
        .common_prefixes()
        .iter()
        .filter_map(|p| p.prefix())
        .filter_map(|p| {
            p.strip_prefix(&prefix) // Remove the platform prefix
                .and_then(|v| v.strip_suffix('/')) // Remove the trailing slash
                .and_then(|v| Version::parse(v).ok())
        })
        .collect::<Vec<Version>>();

    if let Some(latest_version) = versions.iter().max() {
        Ok(format!("{}/{}/", platform, latest_version)) // e.g., "macos/1.0.0/"
    } else {
        Err("No valid versions found".into())
    }
}

