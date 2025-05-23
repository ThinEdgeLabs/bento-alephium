use anyhow::{Context, Result};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use bento_types::{db::new_db_pool, DbPool};
use handler::{BlockApiModule, EventApiModule, TransactionApiModule};
use metrics::Metrics;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{openapi::Info, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
pub mod error;
pub mod handler;
pub mod metrics;

#[derive(Clone, Debug)]
pub struct Config {
    pub db_client: Arc<DbPool>,
    pub api_host: String,
    pub api_port: u16,
}

impl Config {
    pub async fn from_env() -> Result<Self> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_client =
            new_db_pool(&db_url, None).await.context("Failed to create connection pool")?;

        let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
        let port = std::env::var("PORT").unwrap_or("3000".to_string());

        Ok(Self { db_client, api_host: host, api_port: port.parse().unwrap() })
    }

    pub fn api_endpoint(&self) -> String {
        format!("{}:{}", self.api_host, self.api_port)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub metrics: Metrics,
}
use std::str::FromStr;

#[derive(Debug, Clone, Default, Deserialize, ToSchema, Serialize)]
pub struct Pagination {
    #[serde(
        default = "Pagination::default_offset",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub offset: i64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub limit: i64,
}

// Custom deserializer for string to i64 conversion
pub fn deserialize_number_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(0);
    }
    i64::from_str(&s).map_err(serde::de::Error::custom)
}

impl Pagination {
    pub fn get_offset(&self) -> i64 {
        if self.offset < 0 {
            return 0;
        }
        self.offset
    }

    pub fn get_limit(&self) -> i64 {
        if self.limit <= 0 || self.limit > 100 {
            return 10;
        }
        self.limit
    }

    pub fn default_offset() -> i64 {
        0
    }

    pub fn default_limit() -> i64 {
        10
    }
}

pub async fn start(config: Config) -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // create our application state
    let metrics = Metrics::new();
    metrics.set_health_status(true);
    let state = AppState { db: config.clone().db_client, metrics };

    // create our application stack
    let (app, mut api) = configure_api().with_state(state).split_for_parts();

    api.info = Info::new("Alephium REST API", "v1");
    api.info.description = Some("Bento Alephium Indexer REST API".to_string());
    let app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

    let addr = config.api_endpoint();
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello Alephium Indexer API"
}

/// Metrics endpoint that exposes Prometheus metrics
async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response<String>, StatusCode> {
    match state.metrics.encode_metrics() {
        Ok(metrics) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(metrics)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(response)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Setup the API routes
#[allow(clippy::let_and_return)]
pub fn configure_api() -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .nest("/blocks", BlockApiModule::register())
        .nest("/events", EventApiModule::register())
        .nest("/transactions", TransactionApiModule::register())
        .route("/metrics", get(metrics_handler))
        .route("/", get(root));

    // Users can extend with their modules:
    // router.merge(YourApiModule::register())

    router
}
