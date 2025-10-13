use std::path::Path;

use crate::core::{error::PhotoInsightError, image_cache::PhotoInfo};
use std::io::Read;

/// Extracts file_number from a zip archive into memory.
/// Returns  tuple of file name and file contents as Vec<u8>.
pub fn extract_zip_archive(
    image_dir: &str,
    zip_file_name: &str,
    file_number: Vec<usize>,
) -> Result<Vec<(PhotoInfo, Vec<u8>)>, PhotoInsightError> {
    let zip_path = Path::new(image_dir).join(zip_file_name);

    let mut result = Vec::new();
    if zip_path.is_file() {
        let file = std::fs::File::open(&zip_path).map_err(|e| PhotoInsightError::new(e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| PhotoInsightError::new(e))?;

        for idx in &file_number {
            if *idx >= archive.len() {
                return Err(PhotoInsightError::from_message(format!(
                    "File index {} out of bounds in zip {}",
                    idx, zip_file_name
                )));
            }

            let mut file = archive
                .by_index(*idx)
                .map_err(|e| PhotoInsightError::new(e))?;
            let file_name = file.name().to_string();

            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| PhotoInsightError::new(e))?;
            result.push((
                PhotoInfo::new(zip_file_name.to_owned(), file_name, *idx),
                buf,
            ));
        }

        Ok(result)
    } else {
        Err(PhotoInsightError::from_message(
            "Provided zip file path is not a file",
        ))
    }
}

pub fn list_zip_archive(
    image_dir: &str,
    zip_file_name: &str,
) -> Result<Vec<(usize, String)>, PhotoInsightError> {
    let zip_path = Path::new(image_dir).join(zip_file_name);
    let mut image_files = Vec::new();

    if zip_path.is_file() {
        let file = std::fs::File::open(&zip_path).map_err(|e| PhotoInsightError::new(e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| PhotoInsightError::new(e))?;

        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| PhotoInsightError::new(e))?;
            let file_name = file.name().to_string();
            if is_image_file(&file_name) {
                image_files.push((i, file_name));
            }
        }
    } else {
        return Err(PhotoInsightError::from_message(
            "Provided zip file path is not a file",
        ));
    }
    Ok(image_files)
}

pub(crate) fn is_image_file(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png")
}
