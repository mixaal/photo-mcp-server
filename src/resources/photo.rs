use rust_mcp_sdk::schema::{BlobResourceContents, ResourceTemplate};

use crate::{IC, core::error::PhotoInsightError};

pub struct PhotoResource {}

impl PhotoResource {
    pub fn get() -> ResourceTemplate {
        ResourceTemplate {
            annotations: None,
            description: Some("Get photo image as a resource".to_owned()),
            meta: None,
            mime_type: None,
            name: "photo_resource".to_owned(),
            title: Some("Get photo image as a resource".to_owned()),
            uri_template: "{zip_archive}###{photo_file_name}###{offset}###{limit}".to_owned(),
        }
    }

    pub fn read_resource(
        zip_file: String,
        image_file: String,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<BlobResourceContents>, PhotoInsightError> {
        let (infos, _) =
            IC.search_image_by_name(&image_file, &Some(zip_file.clone()), offset, limit);
        let image_data = IC.image_data(infos)?;

        let blobs = image_data
            .iter()
            .map(|(_, mime, image_data)| BlobResourceContents {
                blob: base64::encode(image_data),
                mime_type: Some(mime.clone()),
                meta: None,
                uri: format!("file:///{zip_file}/{image_file}/?offset={offset}&limit={limit}"),
            })
            .collect::<Vec<BlobResourceContents>>();

        Ok(blobs)
    }
}
