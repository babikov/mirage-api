use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    routing::any,
};
use serde_json::json;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::error::Error;

#[derive(Clone)]
struct AppState {
    config: Config,
}

pub async fn run(config: Config) -> Result<(), Error> {
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;

    let state = AppState { config };

    let app = Router::new()
        // Один универсальный маршрут: любой path/method
        .route("/*path", any(handle_request))
        .with_state(state);

    println!("Mirage API listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_request(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // Ищем маршрут по методу и пути
    if let Some(route) = state
        .config
        .routes
        .iter()
        .find(|r| method_matches(&r.method, &method) && r.path == path)
    {
        let body = json!({
            "message": "Mock response from Mirage API",
            "method": method.as_str(),
            "path": path,
            "matched_route": {
                "method": route.method,
                "path": route.path,
            }
        });

        (StatusCode::OK, Json(body))
    } else {
        // ВАЖНО: возвращаем тоже Json, чтобы типы совпали в if/else
        let body = json!({
            "error": format!("No mock found for {} {}", method, path),
            "method": method.as_str(),
            "path": path,
        });

        (StatusCode::NOT_FOUND, Json(body))
    }
}

fn method_matches(cfg_method: &str, req_method: &Method) -> bool {
    cfg_method.eq_ignore_ascii_case(req_method.as_str())
}
