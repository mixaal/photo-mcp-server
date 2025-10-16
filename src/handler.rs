use crate::resources::photo::PhotoResource;
// use crate::tools::fs::FsTools;
use crate::tools::photo::PhotoTools;
use async_trait::async_trait;
use rust_mcp_sdk::schema::{
    BlobResourceContents, ListResourceTemplatesRequest, ListResourceTemplatesResult,
    ReadResourceRequest, ReadResourceResult, ReadResourceResultContentsItem, ResourceTemplate,
    TextResourceContents,
};
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, RpcError,
    schema_utils::CallToolError,
};
use rust_mcp_sdk::{McpServer, mcp_server::ServerHandler};
use std::sync::Arc;
// Custom Handler to handle MCP Messages
pub struct PhotoInsightServerHandler {}

impl PhotoInsightServerHandler {
    pub fn new() -> Self {
        Self {}
    }
}

// To check out a list of all the methods in the trait that you can override, take a look at
// https://github.com/rust-mcp-stack/rust-mcp-sdk/blob/main/crates/rust-mcp-sdk/src/mcp_handlers/mcp_server_handler.rs

#[async_trait]
#[allow(unused)]
impl ServerHandler for PhotoInsightServerHandler {
    // Handle ListToolsRequest, return list of available tools as ListToolsResult
    async fn handle_list_tools_request(
        &self,
        request: ListToolsRequest,
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        // let mut tools = FsTools::tools();
        let mut tools = Vec::new();
        tools.extend(PhotoTools::tools());
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools,
        })
    }

    /// Handles incoming CallToolRequest and processes it using the appropriate tool.
    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        // Attempt to convert request parameters into GreetingTools enum
        // let tool_params = FsTools::try_from(request.params.clone());
        // if tool_params.is_err() {
        // If conversion to GreetingTools fails, try converting to PhotoTools enum
        let photo_tool_params = PhotoTools::try_from(request.params.clone());
        if photo_tool_params.is_err() {
            // If both conversions fail, return an error indicating unknown tool parameters
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown tool parameters: {:?}", request.params),
            )));
        }
        let photo_tool_params = photo_tool_params.unwrap();
        // Match the PhotoTools variant and execute its corresponding logic

        return match photo_tool_params {
            PhotoTools::PhotoExifTool(tool) => tool.call_tool(),
            PhotoTools::PhotoViewByNameTool(tool) => tool.call_tool(),
            PhotoTools::PhotoViewByYearMonthTool(tool) => tool.call_tool(),
            PhotoTools::PhotoSearchByNameTool(tool) => tool.call_tool(),
            PhotoTools::PhotoSearchByYearMonthTool(tool) => tool.call_tool(),
            PhotoTools::PhotoExifTagTool(tool) => tool.call_tool(),
            PhotoTools::PhotoExifSearchTagTool(tool) => tool.call_tool(),
            PhotoTools::ListAllPhotosTool(tool) => tool.call_tool(),
            PhotoTools::PhotoObjectDetectionTool(tool) => tool.call_tool(),
        };
        // } else {
        //     let tool_params = tool_params.unwrap();

        //     // Match the tool variant and execute its corresponding logic
        //     match tool_params {
        //         FsTools::ListFileSystemTool(tool) => tool.call_tool(),
        //         FsTools::ListImagesTool(tool) => tool.call_tool(),
        //     }
        // }
    }

    /// List available resource templates
    async fn handle_list_resource_templates_request(
        &self,
        request: ListResourceTemplatesRequest,
        runtime: Arc<dyn McpServer>,
    ) -> Result<ListResourceTemplatesResult, RpcError> {
        Ok(ListResourceTemplatesResult {
            meta: None,
            next_cursor: None,
            resource_templates: vec![PhotoResource::get()],
        })
    }

    /// Handle Resource Request
    async fn handle_read_resource_request(
        &self,
        request: ReadResourceRequest,
        runtime: Arc<dyn McpServer>,
    ) -> Result<ReadResourceResult, RpcError> {
        println!("request: {request:#?}");
        let uri = request.params.uri;
        let splitted = uri.split("###").collect::<Vec<&str>>();
        if splitted.len() != 4 {
            tracing::error!("invalid params: uri={uri} splitted={splitted:#?}");
            return Err(RpcError::invalid_params());
        }
        let (zip_file, image_file, offset, limit) = (
            splitted[0].to_owned(),
            splitted[1].to_owned(),
            splitted[2],
            splitted[3],
        );
        let offset = offset
            .parse::<usize>()
            .map_err(|e| RpcError::invalid_params().with_message(e.to_string()))?;
        let limit = limit
            .parse::<usize>()
            .map_err(|e| RpcError::invalid_params().with_message(e.to_string()))?;
        let blobs = PhotoResource::read_resource(zip_file, image_file, offset, limit)
            .map_err(|e| RpcError::internal_error().with_message(e.message))?;
        let contents = blobs
            .iter()
            .map(|b| ReadResourceResultContentsItem::BlobResourceContents(b.clone()))
            .collect();
        Ok(ReadResourceResult {
            meta: None,
            contents,
        })
    }
}
