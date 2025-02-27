use anyhow::Result;
use axum::routing::get;
use utoipa::openapi::Info;

use super::AppState;
use crate::api::handler::{BlockApiModule, EventApiModule, TransactionApiModule};
use crate::config::Config;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

pub async fn start(config: Config) -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // create our application state
    let state = AppState { db: config.clone().db_client };

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

/// Setup the API routes
#[allow(clippy::let_and_return)]
pub fn configure_api() -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .nest("/blocks", BlockApiModule::register())
        .nest("/events", EventApiModule::register())
        .nest("/transactions", TransactionApiModule::register())
        .route("/", get(root));

    // Users can extend with their modules:
    // router.merge(YourApiModule::register())

    router
}
