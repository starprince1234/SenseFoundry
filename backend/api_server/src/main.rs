use std::net::SocketAddr;

use search::OpenSearchClient;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let opensearch_url = std::env::var("OPENSEARCH_URL")?;
    let sync_signing_private_key = std::env::var("SYNC_SIGNING_PRIVATE_KEY")?;
    let pool = PgPoolOptions::new()
        .max_connections(
            std::env::var("DATABASE_POOL_MAX")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(20),
        )
        .connect(&database_url)
        .await?;
    let app = api_server::app(
        pool,
        OpenSearchClient::new(&opensearch_url),
        &sync_signing_private_key,
    )?;
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
