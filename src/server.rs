use axum::{routing::{post, any}, Router, Json, http::StatusCode, extract::State, http::HeaderMap, body::Bytes};
use crate::response::JsonResponse;
use std::sync::Arc;
use reqwest;
use serde_json;

pub struct AppState {
    pub url: Option<String>,
    pub verbose: bool,
}

async fn handle_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<JsonResponse>, (StatusCode, Json<JsonResponse>)> {
    if state.verbose {
        eprintln!("Request Headers:");
        for (key, value) in &headers {
            eprintln!("{}: {:?}", key, value);
        }
        eprintln!("\nRequest Body:");
        if let Ok(body_str) = String::from_utf8(body.to_vec()) {
            eprintln!("{}", body_str);
        } else {
            eprintln!("Binary data: {} bytes", body.len());
        }
    }

    match &state.url {
        Some(url) => {
            // 转发请求
            let client = reqwest::Client::new();
            let mut request = client.post(url)
                .body(body);

            // 透传请求头
            for (key, value) in headers {
                if let Some(key) = key {
                    if let Ok(value_str) = value.to_str() {
                        request = request.header(key.as_str(), value_str);
                    }
                }
            }

            let response = request.send().await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(JsonResponse::error(&err.to_string()))))?;

            let body = response.bytes().await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, Json(JsonResponse::error(&err.to_string()))))?;

            if let Ok(json) = serde_json::from_slice(&body) {
                Ok(Json(json))
            } else {
                Ok(Json(JsonResponse::success()))
            }
        },
        None => {
            Ok(Json(JsonResponse::success()))
        }
    }
}

async fn handle_method_not_allowed() -> (StatusCode, Json<JsonResponse>) {
    (StatusCode::METHOD_NOT_ALLOWED, Json(JsonResponse::method_not_allowed()))
}

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(handle_post))
        .route("/", any(handle_method_not_allowed))
        .with_state(state)
}
