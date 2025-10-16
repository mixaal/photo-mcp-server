use rust_mcp_sdk::schema::ImageContent;
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    tool_box,
};

use crate::IC;

const MAX_PHOTO_VIEW_SEARCH_LIMIT: u32 = 50;
const MAX_PHOTO_FILES_SEARCH_LIMIT: u32 = 10000;
const MAX_PHOTO_EXIF_SEARCH_LIMIT: u32 = 1000;
const MAX_PHOTO_YOLO_ANALYZE_LIMIT: u32 = 50;

#[mcp_tool(
    name = "list_all_photos",
    description = "List all photos - accepts offset and limit for pagination, returns list of photo info objects (zip file, index in zip, photo file name) and reference to the next page (next_offset, next_limit) if more results are available"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct ListAllPhotosTool {
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl ListAllPhotosTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let offset = self.offset as usize;
        let limit = self.limit.min(MAX_PHOTO_FILES_SEARCH_LIMIT) as usize;
        tracing::info!("list all images : offset: {offset} Limiting results to {limit}");
        let (infos, total) = IC.list_all_images(offset, limit);

        let next_offset = offset + infos.len();
        let next_limit = limit;

        let json_info = serde_json::json!({
            "result": infos,
            "pagination": {
                "offset": offset,
                "limit": limit,
                "total": total,
                "next_offset": if next_offset < total { Some(next_offset) } else { None },
                "next_limit": next_limit,
            },
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_exif_tags",
    description = "List searchable EXIF tags in the photo collection. You can use these tags with photo_exif_search_tags tool."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoExifTagTool {}
impl PhotoExifTagTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        tracing::info!("photo_exif_tags (list supported exif tags");
        let json_info = serde_json::json!({
            "result": [
                {"name": "width", "type": "Integer", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "height", "type": "Integer", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "month", "type": "Integer", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "year", "type": "Integer", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "aperture", "type": "Float", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "focal_len", "type": "Float", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "iso", "type": "Float", "allowed_operators": ["==", ">", "<", ">=", "<=", "!="]},
                {"name": "shutter_speed", "type": "Float", "allowed_operators": ["!=", "==", ">", "<", ">=", "<=", "!="]},
                {"name": "lens", "type": "String", "allowed_operators": ["!=", "==", "contains", "starts_with", "ends_with"]},
                {"name": "model", "type": "String", "allowed_operators": ["!=", "==", "contains", "starts_with", "ends_with"]},
            ]
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_exif_search_tags",
    description = "Search EXIF tags in the photo collection, returns photo files matching the tag, value and operator. You can use photo_exif_tags tool to get list of searchable tags."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoExifSearchTagTool {
    /// EXIF tag to search for. Example: "model"
    tag: String,
    /// Value to search for. Example: "Canon"
    value: String,
    /// Operator to use for search. Example: "==", "contains", "starts_with", "ends_with", ">", "<", ">=", "<=", "!=" (contains, starts_with, ends_with are allowed only for string tags)
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
        tracing::info!(
            "search_exif_tags: offset={} {} {} {} operator={}",
            self.offset,
            self.limit,
            self.tag,
            self.operator,
            self.value,
        );
        let offset = self.offset as usize;
        let limit = self.limit.min(MAX_PHOTO_EXIF_SEARCH_LIMIT) as usize;
        tracing::info!("search image by EXIF tag : Limiting results to {limit}");
        let (exifs, total) = IC
            .search_image_by_exif_tags(&self.tag, &self.value, &self.operator, offset, limit)
            .map_err(|e| {
                CallToolError::from_message(format!("Failed to search images by EXIF tag: {}", e))
            })?;
        let next_offset = offset + exifs.len();
        let next_limit = limit;

        let json_info = serde_json::json!({
            "query":{
                "tag": self.tag,
                "value": self.value,
                "operator": self.operator,
            },
            "result": exifs,
            "pagination": {
                "offset": offset,
                "limit": limit,
                "total": total,
                "next_offset": if next_offset < total { Some(next_offset) } else { None },
                "next_limit": next_limit,
            },
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
    /// Optionally you can provide zip file name to restrict the search on a given zip file
    /// Example: takeout-20230906T142745Z-050.zip
    zip_file_name: Option<String>,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}
impl PhotoSearchByNameTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        tracing::info!(
            "search image by name: {} {:?} offset={} limit={}",
            self.file_name,
            self.zip_file_name,
            self.offset,
            self.limit
        );
        let offset = self.offset as usize;
        let limit = self.limit.min(MAX_PHOTO_FILES_SEARCH_LIMIT) as usize;
        tracing::info!("search image by name :  Limiting results to {limit}");
        let (infos, total) =
            IC.search_image_by_name(&self.file_name, &self.zip_file_name, offset, limit);
        let next_offset = offset + infos.len();
        let next_limit = limit;
        let json_info = serde_json::json!({
            "query": {"file" : self.file_name },
            "result": infos,
            "pagination": {
                "offset": offset,
                "limit": limit,
                "total": total,
                "next_offset": if next_offset < total { Some(next_offset) } else { None },
                "next_limit": next_limit,
            },
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
        tracing::info!(
            "photo search by year = {}, month={}, offset={}, limit={}",
            self.year,
            self.month,
            self.offset,
            self.limit
        );
        let offset = self.offset as usize;
        let limit = self.limit.min(MAX_PHOTO_FILES_SEARCH_LIMIT) as usize;
        tracing::info!("search image by name : Limiting results to {limit}");
        let (infos, total) = IC.search_image_by_year_month(self.year, self.month, offset, limit);
        let next_offset = offset + infos.len();
        let next_limit = limit;
        let json_info = serde_json::json!({
            "query": {
                "year": self.year,
                "month": self.month,
            },
            "result":  infos,
            "pagination": {
                "offset": offset,
                "limit": limit,
                "total": total,
                "next_offset": if next_offset < total { Some(next_offset) } else { None },
                "next_limit": next_limit,
            },
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
    /// Optionally you can provide zip file name to restrict the search on a given zip file
    /// Example: takeout-20230906T142745Z-050.zip
    zip_file_name: Option<String>,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl PhotoViewByNameTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let limit = self.limit.min(MAX_PHOTO_VIEW_SEARCH_LIMIT) as usize;
        tracing::info!("Limiting results to {}", limit);
        let offset = self.offset as usize;
        let (infos, _) =
            IC.search_image_by_name(&self.file_name, &self.zip_file_name, offset, limit);
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
        let limit = self.limit.min(MAX_PHOTO_VIEW_SEARCH_LIMIT) as usize;
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
    name = "photo_exif_info",
    description = "Accepts photo file name and returns photo meta data (EXIF data) information (can match multiple files if partial name is given or if the photo is in multiple zip files)"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoExifTool {
    /// Photo file name. Can be partial, e.g. "IMG_1234" will match "IMG_1234.jpg", "IMG_1234 (1).jpg", etc.
    /// Example: "IMG_1234.jpg"
    file_name: String,
    /// Optionally you can provide zip file name to restrict the search on a given zip file
    /// Example: takeout-20230906T142745Z-050.zip
    zip_file_name: Option<String>,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl PhotoExifTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        tracing::info!(
            "exif tool: file_name={}, zip_file_name={:?}, offset={}, limit={}",
            self.file_name,
            self.zip_file_name,
            self.offset,
            self.limit
        );
        let offset = self.offset as usize;
        let limit = self.limit.min(MAX_PHOTO_EXIF_SEARCH_LIMIT) as usize;
        tracing::info!("Limiting results to {}", limit);
        let (infos, total) =
            IC.search_image_by_name(&self.file_name, &self.zip_file_name, offset, limit);
        let info_len = infos.len();
        let exifs = IC.exif_info(infos).map_err(|e| {
            CallToolError::from_message(format!("Failed to extract EXIF info: {}", e))
        })?;

        let next_offset = offset + info_len;
        let next_limit = limit;

        let json_info = serde_json::json!({
            "query":{
                "file_name": self.file_name,
            },
            "result": exifs,
            "pagination": {
                "offset": offset,
                "limit": limit,
                "total": total,
                "next_offset": if next_offset < total { Some(next_offset) } else { None },
                "next_limit": next_limit,
            },
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

#[mcp_tool(
    name = "photo_object_detection",
    description = "Accepts photo file name and returns object detections using YOLOv8 (returns vector of images provided, each contains vector of detected objects)"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct PhotoObjectDetectionTool {
    /// Photo file name. Can be partial, e.g. "IMG_1234" will match "IMG_1234.jpg", "IMG_1234 (1).jpg", etc.
    /// Example: "IMG_1234.jpg"
    file_name: String,
    /// Optionally you can provide zip file name to restrict the search on a given zip file
    /// Example: takeout-20230906T142745Z-050.zip
    zip_file_name: Option<String>,
    /// Offset into results
    /// Example: 0
    offset: u32,
    /// Limit number of results returned
    /// Example: 5
    limit: u32,
}

impl PhotoObjectDetectionTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        tracing::info!(
            "photo object detection tool: file_name={}, zip_file_name={:?}, offset={}, limit={}",
            self.file_name,
            self.zip_file_name,
            self.offset,
            self.limit
        );
        let offset = self.offset as usize;
        let limit = self.limit.min(MAX_PHOTO_YOLO_ANALYZE_LIMIT) as usize;
        tracing::info!("Limiting results to {}", limit);
        let (infos, total) =
            IC.search_image_by_name(&self.file_name, &self.zip_file_name, offset, limit);
        let info_len = infos.len();
        let object_detections = IC.yolo_v8_analysis(infos).map_err(|e| {
            CallToolError::from_message(format!("Failed to analyze images using YOLOv8: {}", e))
        })?;

        let next_offset = offset + info_len;
        let next_limit = limit;
        let json_info = serde_json::json!({
            "query":{
                "file_name": self.file_name,
            },
            "result": object_detections,
            "pagination": {
                "offset": offset,
                "limit": limit,
                "total": total,
                "next_offset": if next_offset < total { Some(next_offset) } else { None },
                "next_limit": next_limit,
            },
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            json_info.to_string(),
        )]))
    }
}

tool_box!(
    PhotoTools,
    [
        ListAllPhotosTool,
        PhotoExifTool,
        PhotoViewByNameTool,
        PhotoViewByYearMonthTool,
        PhotoSearchByNameTool,
        PhotoSearchByYearMonthTool,
        PhotoExifTagTool,
        PhotoExifSearchTagTool,
        PhotoObjectDetectionTool
    ]
);
