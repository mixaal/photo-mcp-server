use std::sync::Arc;
use std::time::Duration;

use rust_mcp_sdk::event_store::InMemoryEventStore;
use rust_mcp_sdk::mcp_server::{HyperServerOptions, hyper_server};

use crate::handler::PhotoInsightServerHandler;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
    ServerCapabilitiesTools,
};

use rust_mcp_sdk::{error::SdkResult, mcp_server::ServerHandler};

pub struct AppStateX<H: ServerHandler> {
    pub server_details: InitializeResult,
    pub handler: H,
}

pub async fn start_server() -> SdkResult<()> {
    // STEP 1: Define server details and capabilities
    let server_details = InitializeResult {
        // server name and version
        server_info: Implementation {
            name: "PhotoTool  MCP Server SSE".to_string(),
            version: "0.1.0".to_string(),
            title: Some("PhotoTool Organizer, Insight helper".to_string()),
        },
        capabilities: ServerCapabilities {
            // indicates that server support mcp tools
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default() // Using default values for other fields
        },
        meta: None,
        instructions: Some(
            vec![
                "This server provides help on deeper understanding of the photo collection we have in our zip files.",
                "It provides methods for listing of photos, describing exif tags of photos, searching of photos by name, exif tags or by year and month.",
                "Most responses contain pagination to help browse on results (next_offset, next_limit). If next_offset is not present or null, there are no more pages.",
                "Image file description contains zip_file - that is the file on the file system, image_file - \
the file inside the zip archive and image_index_in_zip which describes the \
archive index number in the zip (for fast extraction).",
                "There are also helpers on viewing photos that send the ImageContent (base64 \
encoded). Those methods do not have pagination but offset and limit can be used and derived from non-view methods.",
            ]
            .join("\n"),
        ),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // STEP 2: instantiate our custom handler for handling MCP messages
    let handler = PhotoInsightServerHandler::new();

    // STEP 3: instantiate HyperServer, providing `server_details` , `handler` and HyperServerOptions
    let server = hyper_server::create_server(
        server_details,
        handler,
        HyperServerOptions {
            enable_ssl: true,
            ssl_cert_path: Some("certs/server.crt".to_owned()),
            ssl_key_path: Some("certs/server.key".to_owned()),
            sse_support: false,
            host: "0.0.0.0".to_string(),
            ping_interval: Duration::from_secs(5),
            event_store: Some(Arc::new(InMemoryEventStore::default())), // enable resumability
            ..Default::default()
        },
    );

    // tracing::info!("{}", server.server_info(None).await?);

    // STEP 4: Start the server
    server.start().await?;
    Ok(())
}
