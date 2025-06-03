use anyhow::{Context, Result};
use axum::routing::get;
use bento_types::{db::new_db_pool, DbPool};
use handler::{BlockApiModule, EventApiModule, TransactionApiModule};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{openapi::Info, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

pub mod error;
pub mod handler;

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

pub async fn start(config: Config, custom_router: Option<OpenApiRouter<AppState>>) -> Result<()> {
    let state = AppState { db: config.clone().db_client };

    let (app, mut api) = configure_api(custom_router).with_state(state).split_for_parts();

    api.info = Info::new("REST API", "v1");
    api.info.description = Some("Bento Alephium Indexer REST API".to_string());
    let app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

    let addr = config.api_endpoint();
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello Alephium Indexer API"
}

#[allow(clippy::let_and_return)]
pub fn configure_api(custom_router: Option<OpenApiRouter<AppState>>) -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .nest("/blocks", BlockApiModule::register())
        .nest("/events", EventApiModule::register())
        .nest("/transactions", TransactionApiModule::register())
        .route("/", get(root));

    if let Some(custom_router) = custom_router {
        router.merge(custom_router)
    } else {
        router
    }
}
