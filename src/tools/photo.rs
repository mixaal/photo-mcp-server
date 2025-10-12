use rust_mcp_sdk::schema::ImageContent;
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    tool_box,
};

use crate::IC;

#[mcp_tool(
    name = "photo_exif_tags",
    description = "List searchable EXIF tags in the photo collection"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoExifTagTool {}
impl PhotoExifTagTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let json_info = serde_json::json!({
            "tags": [
                "width",
                "height",
                "month",
                "year",
                "aperture",
                "focal_len",
                "iso",
                "lens",
                "model",
                "shutter_speed"
            ]
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_exif_search_tags",
    description = "Search EXIF tags in the photo collection"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoExifSearchTagTool {
    /// EXIF tag to search for. Example: "model"
    tag: String,
    /// Value to search for. Example: "Canon"
    value: String,
    /// Operator to use for search. Example: "==", "contains", "starts_with", "ends_with", ">", "<", ">=", "<=", "!="
    operator: String,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}
impl PhotoExifSearchTagTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let offset = self.offset as usize;
        let limit = self.limit.min(50) as usize;
        tracing::info!("search image by EXIF tag : offset: {offset} Limiting results to {limit}");
        let (infos, total) = IC
            .search_image_by_exif_tags(&self.tag, &self.value, &self.operator, offset, limit)
            .map_err(|e| {
                CallToolError::from_message(format!("Failed to search images by EXIF tag: {}", e))
            })?;
        let next_offset = offset + infos.len();
        let next_limit = limit;

        let json_info = serde_json::json!({
            "query_tag": self.tag,
            "query_value": self.value,
            "query_operator": self.operator,
            "matches": infos,
            "total_match_count": total,
            "next_offset": if next_offset < total { Some(next_offset) } else { None },
            "next_limit": next_limit,
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_search_by_name",
    description = "Accepts photo file name and returns photo files matching the file_name"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoSearchByNameTool {
    /// Photo file name. Can be partial, e.g. "IMG_1234" will match "IMG_1234.jpg", "IMG_1234 (1).jpg", etc.
    /// Example: "IMG_1234.jpg"
    file_name: String,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}
impl PhotoSearchByNameTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let offset = self.offset as usize;
        let limit = self.limit.min(50) as usize;
        tracing::info!("search image by name : offset: {offset} Limiting results to {limit}");
        let (infos, total) = IC.search_image_by_name(&self.file_name, offset, limit);
        let next_offset = offset + infos.len();
        let next_limit = limit;
        let json_info = serde_json::json!({
            "query_file_name": self.file_name,
            "matches": infos,
            "total_match_count": total,
            "next_offset": if next_offset < total { Some(next_offset) } else { None },
            "next_limit": next_limit,
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_search_by_year_month",
    description = "Accepts year and month and returns photo files matching the name"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoSearchByYearMonthTool {
    /// Year of the photo. Example: 2021
    year: u32,
    /// Month of the photo. Example: 1 for January, 12 for December
    month: u32,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}
impl PhotoSearchByYearMonthTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let offset = self.offset as usize;
        let limit = self.limit.min(50) as usize;
        tracing::info!("search image by name : offset: {offset} Limiting results to {limit}");
        let (infos, total) = IC.search_image_by_year_month(self.year, self.month, offset, limit);
        let next_offset = offset + infos.len();
        let next_limit = limit;
        let json_info = serde_json::json!({
            "query_year": self.year,
            "query_month": self.month,
            "matches": infos,
            "total_match_count": total,
            "next_offset": if next_offset < total { Some(next_offset) } else { None },
            "next_limit": next_limit,
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_view_by_name",
    description = "Accepts photo file name and returns photo image data"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoViewByNameTool {
    /// Photo file name. Can be partial, e.g. "IMG_1234" will match "IMG_1234.jpg", "IMG_1234 (1).jpg", etc.
    /// Example: "IMG_1234.jpg"
    file_name: String,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl PhotoViewByNameTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let limit = self.limit.min(50) as usize;
        tracing::info!("Limiting results to {}", limit);
        let offset = self.offset as usize;
        let (infos, _) = IC.search_image_by_name(&self.file_name, offset, limit);
        let image_data = IC
            .image_data(infos)
            .map_err(|e| {
                CallToolError::from_message(format!("Failed to extract image data: {}", e))
            })?
            .iter()
            .map(|(file_name, mime, data)| {
                ImageContent::new(
                    base64::encode(data),
                    mime.clone(),
                    None,
                    Some(
                        serde_json::json!({"name":file_name})
                            .as_object()
                            .cloned()
                            .unwrap(),
                    ),
                )
            })
            .collect();

        Ok(CallToolResult::image_content(image_data))
    }
}

#[mcp_tool(
    name = "photo_view_by_year_month",
    description = "Accepts year and month  and returns photo image data"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoViewByYearMonthTool {
    /// Year of the photo. Example: 2021
    year: u32,
    /// Month of the photo. Example: 1 for January, 12 for December
    month: u32,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl PhotoViewByYearMonthTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let limit = self.limit.min(50) as usize;
        tracing::info!("Limiting results to {}", limit);
        let offset = self.offset as usize;
        let (infos, _) = IC.search_image_by_year_month(self.year, self.month, offset, limit);
        let image_data = IC
            .image_data(infos)
            .map_err(|e| {
                CallToolError::from_message(format!("Failed to extract image data: {}", e))
            })?
            .iter()
            .map(|(file_name, mime, data)| {
                ImageContent::new(
                    base64::encode(data),
                    mime.clone(),
                    None,
                    Some(
                        serde_json::json!({"name":file_name})
                            .as_object()
                            .cloned()
                            .unwrap(),
                    ),
                )
            })
            .collect();

        Ok(CallToolResult::image_content(image_data))
    }
}

#[mcp_tool(
    name = "photo_info",
    description = "Accepts photo file name and returns photo meta data (EXIF data) information"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoInfoTool {
    /// Photo file name. Can be partial, e.g. "IMG_1234" will match "IMG_1234.jpg", "IMG_1234 (1).jpg", etc.
    /// Example: "IMG_1234.jpg"
    file_name: String,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl PhotoInfoTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let offset = self.offset as usize;
        let limit = self.limit.min(50) as usize;
        tracing::info!("Limiting results to {}", limit);
        let (infos, total) = IC.search_image_by_name(&self.file_name, 0, limit);
        let info_len = infos.len();
        let exifs = IC.exif_info(infos).map_err(|e| {
            CallToolError::from_message(format!("Failed to extract EXIF info: {}", e))
        })?;

        let next_offset = offset + info_len;
        let next_limit = limit;
        let json_info = serde_json::json!({
            "query_file_name": self.file_name,
            "matches": exifs,
            "total_match_count": total,
            "next_offset": if next_offset < total { Some(next_offset) } else { None },
            "next_limit": next_limit,
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

tool_box!(
    PhotoTools,
    [
        PhotoInfoTool,
        PhotoViewByNameTool,
        PhotoViewByYearMonthTool,
        PhotoSearchByNameTool,
        PhotoSearchByYearMonthTool,
        PhotoExifTagTool,
        PhotoExifSearchTagTool
    ]
);
