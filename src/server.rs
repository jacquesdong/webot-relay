use axum::{routing::{post, any}, Router, http::StatusCode, extract::State, http::HeaderMap, body::Bytes, response::Response, body::Body, http::header};
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
) -> Result<Response, (StatusCode, Response)> {
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
                .map_err(|err| {
                    let json = JsonResponse::error(&err.to_string());
                    let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    response.headers_mut().insert(header::SERVER, axum::http::HeaderValue::from_static("webot-relay"));
                    response.headers_mut().insert(header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/json"));
                    (StatusCode::INTERNAL_SERVER_ERROR, response)
                })?;

            if state.verbose {
                eprintln!("Response Status: {}", response.status());
                eprintln!("Response Headers:");
                for (key, value) in response.headers() {
                    eprintln!("{}: {:?}", key, value);
                }
            }

            let body = response.bytes().await
                .map_err(|err| {
                    let json = JsonResponse::error(&err.to_string());
                    let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    response.headers_mut().insert(header::SERVER, axum::http::HeaderValue::from_static("webot-relay"));
                    response.headers_mut().insert(header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/json"));
                    (StatusCode::INTERNAL_SERVER_ERROR, response)
                })?;

            if state.verbose {
                eprintln!("\nResponse Body:");
                if let Ok(body_str) = String::from_utf8(body.to_vec()) {
                    eprintln!("{}", body_str);
                } else {
                    eprintln!("Binary data: {} bytes", body.len());
                }
            }

            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
                *response.status_mut() = StatusCode::OK;
                response.headers_mut().insert(header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/json"));
                Ok(response)
            } else {
                let json = JsonResponse::success();
                let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
                *response.status_mut() = StatusCode::OK;
                response.headers_mut().insert(header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/json"));
                Ok(response)
            }
        },
        None => {
            let json = JsonResponse::url_not_configured();
            let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
            *response.status_mut() = StatusCode::NOT_FOUND;
            response.headers_mut().insert(header::SERVER, axum::http::HeaderValue::from_static("webot-relay"));
            response.headers_mut().insert(header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/json"));
            Err((StatusCode::NOT_FOUND, response))
        }
    }
}

async fn handle_method_not_allowed() -> Response {
    let json = JsonResponse::method_not_allowed();
    let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
    response.headers_mut().insert(header::SERVER, axum::http::HeaderValue::from_static("webot-relay"));
    response.headers_mut().insert(header::CONTENT_TYPE, axum::http::HeaderValue::from_static("application/json"));
    response
}

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(handle_post))
        .route("/", any(handle_method_not_allowed))
        .with_state(state)
}
