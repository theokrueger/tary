//! HTTP server input source.
//! Serves a simple HTML page on '/' and provides a /todo API for creating todos.

use crate::config::Config;
use crate::content::{Content, ContentType};
use crate::sources::TarySource;
use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use log::{error, info, trace};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

const DEFAULT_HTML: &str = include_str!("./index.html");

/// State shared across axum request handlers
struct AppState {
    tx: Sender<Content>,
    html_path: Option<String>,
}

/// JSON body for creating a todo via POST /todo
#[derive(Deserialize)]
struct TodoRequest {
    content: String,
    due: Option<String>,
    dest: Option<String>,
}

pub struct HttpServerSource {
    cfg: Arc<Config>,
}

impl TarySource for HttpServerSource {
    async fn listen(self, tx: Sender<Content>) {
        let http = self.cfg.sources.http_server.as_ref();
        let html_path = http.and_then(|h| h.html_path.clone());

        let host = http
            .map(|h| h.host.clone())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = http.map(|h| h.port).unwrap_or(3000);

        let state = Arc::new(AppState { tx, html_path });

        let app = Router::new()
            .route("/", get(serve_html))
            .route("/todo", post(create_todo))
            .with_state(state);

        let addr = format!("{host}:{port}");
        info!("Starting HTTP server on {addr}");

        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind HTTP server to {addr}: {e}");
                return;
            }
        };

        let _ = axum::serve(listener, app).await;
    }

    fn init(cfg: Arc<Config>) -> Option<Box<Self>> {
        let http = cfg.sources.http_server.as_ref()?;
        if http.enabled {
            Some(Box::new(Self { cfg }))
        } else {
            None
        }
    }
}

/// Serve the HTML page on '/'
async fn serve_html(State(state): State<Arc<AppState>>) -> Response {
    if let Some(path) = &state.html_path
        && let Ok(html) = tokio::fs::read_to_string(path).await
    {
        return Html(html).into_response();
    }
    Html(DEFAULT_HTML).into_response()
}

/// Create a new todo via POST /todo
async fn create_todo(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TodoRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    trace!("Received todo via HTTP: {}", payload.content);

    let mut content = Content::new(ContentType::Todo, "http".to_string(), None);
    content.content = payload.content;
    content.dest = payload.dest;

    if let Some(due_str) = payload.due
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&due_str)
    {
        content.due = Some(dt.with_timezone(&chrono::Local));
    }

    let _ = state.tx.send(content);

    Ok(StatusCode::CREATED)
}
