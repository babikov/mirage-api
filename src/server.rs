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
use std::collections::HashMap;
use tokio::net::TcpListener;

use crate::config::{MediaType, OpenApi, Operation, PathItem, Response, Schema};
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
    let query_params = parse_query(req.uri().query());

    // Find PathItem by path template, supporting /users/{id}
    if let Some((_, path_item)) = state
        .spec
        .paths
        .iter()
        .find(|(template, _)| match_path(template, &path))
    {
        if let Some(operation) = find_operation_for_method(path_item, &method) {
            if let Some((status, body, content_type)) =
                build_response_from_operation(operation, &query_params)
            {
                return build_response(status, body, content_type);
            }
        }
    }

    build_not_found_response(method, path)
}

/// Very simple query parser: ?a=1&b=2 → HashMap { "a": "1", "b": "2" }
fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(q) = query {
        for pair in q.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut iter = pair.splitn(2, '=');
            let key = iter.next().unwrap_or_default();
            let value = iter.next().unwrap_or_default();
            map.insert(key.to_string(), value.to_string());
        }
    }

    map
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

fn match_path(template: &str, actual: &str) -> bool {
    // Remove trailing / so that /users and /users/ are considered the same
    let t = template.trim_end_matches('/');
    let a = actual.trim_end_matches('/');

    let t_parts: Vec<_> = t.split('/').filter(|s| !s.is_empty()).collect();
    let a_parts: Vec<_> = a.split('/').filter(|s| !s.is_empty()).collect();

    if t_parts.len() != a_parts.len() {
        return false;
    }

    for (t_seg, a_seg) in t_parts.iter().zip(a_parts.iter()) {
        if is_path_param(t_seg) {
            // {id} matches any non-empty segment
            if a_seg.is_empty() {
                return false;
            }
            continue;
        }

        if t_seg != a_seg {
            return false;
        }
    }

    true
}

fn is_path_param(seg: &str) -> bool {
    seg.starts_with('{') && seg.ends_with('}') && seg.len() > 2
}

#[allow(clippy::collapsible_if)]
fn build_response_from_operation(
    operation: &Operation,
    query: &HashMap<String, String>,
) -> Option<(u16, Option<BodyKind>, String)> {
    if operation.responses.is_empty() {
        return None;
    }

    // 1) Try to use 200
    if let Some(resp) = operation.responses.get("200") {
        if let Some(res) = build_body_from_response(resp, query) {
            return Some((200, res.0, res.1));
        }
    }

    // 2) Otherwise take the first available status code
    if let Some((status_code, resp)) = operation.responses.iter().next() {
        let status = status_code.parse::<u16>().unwrap_or(200);
        if let Some(res) = build_body_from_response(resp, query) {
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

/// Главная логика выбора примера:
/// 1) Если в media-type есть x-mirage-example-param: param,
///    и в query есть ?param=key, и есть examples.key → взять его.
/// 2) Иначе example.
/// 3) Иначе первый из examples.
/// 4) Иначе None → дальше разрулит schema.
fn pick_example(mt: &MediaType, query: &HashMap<String, String>) -> Option<Value> {
    // 1) x-mirage-example-param
    if let Some(param) = &mt.example_param {
        if let Some(key) = query.get(param) {
            if let Some(ex) = mt.examples.get(key) {
                if let Some(v) = &ex.value {
                    return Some(v.clone());
                }
            }
        }
    }

    // 2) Single example
    if let Some(ex) = &mt.example {
        return Some(ex.clone());
    }

    // 3) First from examples
    for ex in mt.examples.values() {
        if let Some(value) = &ex.value {
            return Some(value.clone());
        }
    }

    // 4) No examples
    None
}

fn generate_from_schema(schema: &Schema) -> Value {
    // 1) If there is an enum — take the first value
    if !schema.enum_values.is_empty() {
        return schema.enum_values[0].clone();
    }

    let ty = schema.ty.as_deref().unwrap_or("object");

    match ty {
        "string" => {
            if let Some(format) = &schema.format {
                match format.as_str() {
                    "date-time" => Value::String("2025-01-01T00:00:00Z".to_string()),
                    "date" => Value::String("2025-01-01".to_string()),
                    "uuid" => Value::String("00000000-0000-0000-0000-000000000000".to_string()),
                    _ => Value::String(format!("string({})", format)),
                }
            } else {
                Value::String("string".to_string())
            }
        }
        "number" => Value::Number(serde_json::Number::from_f64(123.45).unwrap()),
        "integer" => Value::Number(serde_json::Number::from(123)),
        "boolean" => Value::Bool(true),
        "array" => {
            if let Some(item_schema) = &schema.items {
                Value::Array(vec![generate_from_schema(item_schema)])
            } else {
                Value::Array(vec![])
            }
        }
        "object" => {
            if !schema.properties.is_empty() {
                let mut map = serde_json::Map::new();
                for (name, prop_schema) in &schema.properties {
                    map.insert(name.clone(), generate_from_schema(prop_schema));
                }
                Value::Object(map)
            } else {
                Value::Object(serde_json::Map::new())
            }
        }
        _ => Value::Object(serde_json::Map::new()),
    }
}

#[allow(clippy::collapsible_if)]
fn build_body_from_response(
    resp: &Response,
    query: &HashMap<String, String>,
) -> Option<(Option<BodyKind>, String)> {
    if resp.content.is_empty() {
        return Some((None, "text/plain".to_string()));
    }

    // 1) First try JSON: example / examples / schema
    if let Some(mt) = resp.content.get("application/json") {
        // example / examples (+ x-mirage-example-param)
        if let Some(example) = pick_example(mt, query) {
            return Some((
                Some(BodyKind::Json(example)),
                "application/json".to_string(),
            ));
        }

        // schema without example/examples → generate a mock
        if let Some(schema) = &mt.schema {
            let value = generate_from_schema(schema);
            return Some((Some(BodyKind::Json(value)), "application/json".to_string()));
        }
    }

    // 2) Then any other content-type
    if let Some((content_type, mt)) = resp.content.iter().next() {
        // example / examples (+ x-mirage-example-param)
        if let Some(example) = pick_example(mt, query) {
            if let Some(s) = example.as_str() {
                return Some((Some(BodyKind::Text(s.to_string())), content_type.clone()));
            } else {
                return Some((Some(BodyKind::Json(example)), content_type.clone()));
            }
        } else {
            // For now we ignore schema for non-JSON types and return an empty body
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
