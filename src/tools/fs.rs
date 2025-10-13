use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    tool_box,
};
use std::fs;
use std::path::Path;

use crate::core;

#[mcp_tool(
    name = "list_images_in_zip_archive",
    description = "Accepts zip archive path and lists all images within that archive"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct ListImagesTool {
    /// Image directory path.
    /// Example: "/home/user/photos/"
    image_dir: String,
    /// Zip archive file name.
    /// Example: "photos.zip"
    zip_file_name: String,
}

impl ListImagesTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let zip_path = Path::new(&self.image_dir).join(&self.zip_file_name);
        let mut image_files = Vec::new();

        if zip_path.is_file() {
            let file = fs::File::open(&zip_path).map_err(|e| CallToolError::new(e))?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| CallToolError::new(e))?;

            for i in 0..archive.len() {
                let file = archive.by_index(i).map_err(|e| CallToolError::new(e))?;
                let file_name = file.name().to_string();
                let lower = file_name.to_lowercase();
                if lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".png")
                    || lower.ends_with(".gif")
                {
                    image_files.push(file_name);
                }
            }
        } else {
            return Err(CallToolError::from_message(
                "Provided zip file path is not a file",
            ));
        }

        let json_info = serde_json::json!({
            "image_files": image_files,
            "number_of_images": image_files.len(),
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "list_file_system",
    description = "Accepts image directory path and lists all image zip archive files in that directory"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct ListFileSystemTool {
    /// Image directory path.
    /// Example: "/home/user/photos/"
    image_dir: String,
}

impl ListFileSystemTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let zip_files = core::traversal::list_directory_zip_files(&self.image_dir)
            .map_err(|e| CallToolError::new(e))?;

        let json_info = serde_json::json!({
            "zip_files": zip_files,
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}
// we don't need this
// tool_box!(FsTools, [ListFileSystemTool, ListImagesTool]);
