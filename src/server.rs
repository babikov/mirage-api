use axum::http::header::{HeaderName, HeaderValue};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    routing::any,
};
use serde_json::Value;
use tokio::net::TcpListener;

use crate::config::{OpenApi, Operation, PathItem, Response};
use crate::error::Error;

#[derive(Clone)]
struct AppState {
    spec: OpenApi,
}

pub async fn run(spec: OpenApi, addr: String) -> Result<(), Error> {
    let listener = TcpListener::bind(&addr).await?;

    let state = AppState { spec };

    let app = Router::new()
        // Single catch-all route: any path/method
        .route("/*path", any(handle_request))
        .with_state(state);

    println!("Mirage API listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

#[allow(clippy::collapsible_if)]
async fn handle_request(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // Look up PathItem by exact path
    if let Some(path_item) = state.spec.paths.get(&path) {
        if let Some(operation) = find_operation_for_method(path_item, &method) {
            if let Some((status, body, content_type)) = build_response_from_operation(operation) {
                return build_response(status, body, content_type);
            }
        }
    }

    build_not_found_response(method, path)
}

fn find_operation_for_method<'a>(
    path_item: &'a PathItem,
    method: &Method,
) -> Option<&'a Operation> {
    match *method {
        Method::GET => path_item.get.as_ref(),
        Method::POST => path_item.post.as_ref(),
        Method::PUT => path_item.put.as_ref(),
        Method::DELETE => path_item.delete.as_ref(),
        Method::PATCH => path_item.patch.as_ref(),
        _ => None,
    }
}

#[allow(clippy::collapsible_if)]
fn build_response_from_operation(operation: &Operation) -> Option<(u16, Option<BodyKind>, String)> {
    if operation.responses.is_empty() {
        return None;
    }

    // 1) Try to use 200
    if let Some(resp) = operation.responses.get("200") {
        if let Some(res) = build_body_from_response(resp) {
            return Some((200, res.0, res.1));
        }
    }

    // 2) Otherwise take the first available status code
    if let Some((status_code, resp)) = operation.responses.iter().next() {
        let status = status_code.parse::<u16>().unwrap_or(200);
        if let Some(res) = build_body_from_response(resp) {
            return Some((status, res.0, res.1));
        }
    }

    None
}

#[derive(Debug)]
enum BodyKind {
    Json(Value),
    Text(String),
}

#[allow(clippy::collapsible_if)]
fn build_body_from_response(resp: &Response) -> Option<(Option<BodyKind>, String)> {
    if resp.content.is_empty() {
        return Some((None, "text/plain".to_string()));
    }

    // First try to use JSON
    if let Some(mt) = resp.content.get("application/json") {
        if let Some(example) = &mt.example {
            return Some((
                Some(BodyKind::Json(example.clone())),
                "application/json".to_string(),
            ));
        }
    }

    // Then fall back to any other content-type
    if let Some((content_type, mt)) = resp.content.iter().next() {
        if let Some(example) = &mt.example {
            // If example is a string → treat as text
            if let Some(s) = example.as_str() {
                return Some((Some(BodyKind::Text(s.to_string())), content_type.clone()));
            } else {
                return Some((Some(BodyKind::Json(example.clone())), content_type.clone()));
            }
        } else {
            // No example → empty body
            return Some((None, content_type.clone()));
        }
    }

    None
}

fn build_response(
    status: u16,
    body_kind: Option<BodyKind>,
    content_type: String,
) -> axum::response::Response {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);

    // Builder and headers are handled here
    let mut builder = axum::http::Response::builder().status(status);
    {
        let headers = builder.headers_mut().expect("headers_mut failed");
        let name = HeaderName::from_static("content-type");
        if let Ok(value) = HeaderValue::try_from(content_type.as_str()) {
            headers.insert(name, value);
        }
    }

    let body_bytes = match body_kind {
        Some(BodyKind::Json(value)) => match serde_json::to_vec(&value) {
            Ok(bytes) => bytes,
            Err(_) => b"{}".to_vec(),
        },
        Some(BodyKind::Text(s)) => s.into_bytes(),
        None => Vec::new(),
    };

    builder
        .body(Body::from(body_bytes))
        .expect("failed to build response")
}

fn build_not_found_response(method: Method, path: String) -> axum::response::Response {
    let status = StatusCode::NOT_FOUND;
    let body_string = format!("No mock found for {} {}", method, path);

    axum::http::Response::builder()
        .status(status)
        .body(Body::from(body_string))
        .expect("failed to build 404 response")
}
