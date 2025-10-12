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

    // let infos = image_cache.search_image("20210125_134310-upraveno.jpg");
    let infos = IC.search_image_by_name("Michal, Edita/20210125_134310-upraveno.jpg", 0, 20);

    tracing::info!("Search results: {:#?}", infos);

    server::start_server().await?;

    Ok(())
}
