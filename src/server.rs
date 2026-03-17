use crate::response::JsonResponse;
use axum::{
    Router,
    body::Body,
    body::Bytes,
    extract::State,
    http::HeaderMap,
    http::StatusCode,
    http::header,
    response::Response,
    routing::{any, post},
};
use chrono;
use reqwest;
use serde_json;
use std::sync::Arc;

const SERVER_NAME: &str = "webot-relay";
const CONTENT_TYPE: &str = "application/json";

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
        eprintln!(">>> Request Headers:");
        for (key, value) in &headers {
            eprintln!("{}: {:?}", key, value);
        }
        eprintln!();
        eprintln!(">>> Request Body:");
        if let Ok(body_str) = String::from_utf8(body.to_vec()) {
            eprintln!("{}", body_str);
        } else {
            eprintln!("Binary data: {} bytes", body.len());
            eprintln!("{:?}", body);
        }
    }

    match &state.url {
        Some(url) => {
            // 转发请求
            let client = reqwest::Client::new();
            let mut request = client.post(url).body(body);

            // 透传请求头，移除 Host 字段
            for (key, value) in headers {
                if let Some(key) = key {
                    if key != header::HOST {
                        if let Ok(value_str) = value.to_str() {
                            request = request.header(key.as_str(), value_str);
                        }
                    }
                }
            }

            let response = request.send().await.map_err(|err| {
                let json = JsonResponse::error(&err.to_string());
                let error_response = serde_json::to_string(&json).unwrap();

                if state.verbose {
                    eprintln!("*** {}", error_response);
                }

                let mut response = Response::new(Body::from(error_response));
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response.headers_mut().insert(
                    header::SERVER,
                    axum::http::HeaderValue::from_static(SERVER_NAME),
                );
                response.headers_mut().insert(
                    header::DATE,
                    axum::http::HeaderValue::from_str(&chrono::Utc::now().to_rfc2822()).unwrap(),
                );
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static(CONTENT_TYPE),
                );
                (StatusCode::INTERNAL_SERVER_ERROR, response)
            })?;

            if state.verbose {
                eprintln!("<<< {}", response.status());
                eprintln!("<<< Response Headers:");
                for (key, value) in response.headers() {
                    eprintln!("{}: {:?}", key, value);
                }
            }

            let body = response.bytes().await.map_err(|err| {
                let json = JsonResponse::error(&err.to_string());
                let error_response = serde_json::to_string(&json).unwrap();

                if state.verbose {
                    eprintln!("*** {}", error_response);
                }

                let mut response = Response::new(Body::from(error_response));
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response.headers_mut().insert(
                    header::SERVER,
                    axum::http::HeaderValue::from_static(SERVER_NAME),
                );
                response.headers_mut().insert(
                    header::DATE,
                    axum::http::HeaderValue::from_str(&chrono::Utc::now().to_rfc2822()).unwrap(),
                );
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static(CONTENT_TYPE),
                );
                (StatusCode::INTERNAL_SERVER_ERROR, response)
            })?;

            if state.verbose {
                eprintln!();
                eprintln!("<<< Response Body:");
                if let Ok(body_str) = String::from_utf8(body.to_vec()) {
                    eprintln!("{}", body_str);
                } else {
                    eprintln!("Binary data: {} bytes", body.len());
                    eprintln!("{:?}", body);
                }
            }

            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
                *response.status_mut() = StatusCode::OK;
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static(CONTENT_TYPE),
                );
                Ok(response)
            } else {
                let json = JsonResponse::success();
                let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
                *response.status_mut() = StatusCode::OK;
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static(CONTENT_TYPE),
                );
                Ok(response)
            }
        }
        None => {
            let json = JsonResponse::url_not_configured();
            let error_response = serde_json::to_string(&json).unwrap();

            if state.verbose {
                eprintln!("*** {}", error_response);
            }

            let mut response = Response::new(Body::from(error_response));
            *response.status_mut() = StatusCode::OK;
            response.headers_mut().insert(
                header::SERVER,
                axum::http::HeaderValue::from_static(SERVER_NAME),
            );
            response.headers_mut().insert(
                header::DATE,
                axum::http::HeaderValue::from_str(&chrono::Utc::now().to_rfc2822()).unwrap(),
            );
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(CONTENT_TYPE),
            );
            Err((StatusCode::OK, response))
        }
    }
}

async fn handle_method_not_allowed() -> Response {
    let json = JsonResponse::method_not_allowed();
    let mut response = Response::new(Body::from(serde_json::to_vec(&json).unwrap()));
    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
    response.headers_mut().insert(
        header::SERVER,
        axum::http::HeaderValue::from_static(SERVER_NAME),
    );
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(CONTENT_TYPE),
    );
    response
}

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(handle_post))
        .route("/", any(handle_method_not_allowed))
        .with_state(state)
}
