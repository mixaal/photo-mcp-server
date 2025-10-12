use crate::core::error::PhotoInsightError;

pub fn list_directory_zip_files(dir_path: &str) -> Result<Vec<String>, PhotoInsightError> {
    use std::fs;
    use std::path::Path;

    let dir_path = Path::new(dir_path);
    let mut zip_files = Vec::new();

    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path).map_err(|e| PhotoInsightError::new(e))? {
            let entry = entry.map_err(|e| PhotoInsightError::new(e))?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "zip" {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        zip_files.push(file_name.to_string());
                    }
                }
            }
        }
    } else {
        return Err(PhotoInsightError::from_message(
            "Provided path is not a directory",
        ));
    }
    Ok(zip_files)
}
