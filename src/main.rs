use photo_mcp_server::{IC, server};
use rust_mcp_sdk::error::SdkResult;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> SdkResult<()> {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    rustls::crypto::ring::default_provider().install_default();

    let _ = IC.search_image_by_name(&".".to_owned(), &None, 0, 20);

    server::start_server().await?;

    Ok(())
}
