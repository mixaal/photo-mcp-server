use serde::Serialize;

use crate::core::{error::PhotoInsightError, image_cache::PhotoInfo};

#[derive(Debug, Serialize)]
pub struct DetectedObject {
    pub class_name: String,
    pub confidence: f32,
    pub bbox: (f32, f32, f32, f32), // (xmin, ymin, xmax, ymax)
}

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    photo_info: PhotoInfo,
    object_detection: Vec<DetectedObject>,
}

impl AnalysisResult {
    fn new(photo_info: PhotoInfo, object_detection: Vec<DetectedObject>) -> Self {
        Self {
            photo_info,
            object_detection,
        }
    }
}

pub fn analyze_images_using_yolo(
    images: Vec<(PhotoInfo, Vec<u8>)>,
) -> Result<Vec<AnalysisResult>, PhotoInsightError> {
    use rand::Rng;
    use rand::distr::Alphanumeric;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use yolo_v8::YoloV8ObjectDetection;

    let yolo = YoloV8ObjectDetection::new();

    let mut results = Vec::new();
    for (photo_info, image_data) in images {
        // Create a temporary file path
        let mut temp_path = env::temp_dir();
        let rand_str: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        temp_path.push(format!("{}_yolo_tmp.jpg", rand_str));

        // Write image_data to the temporary file
        let mut temp_file = File::create(&temp_path).map_err(|e| {
            PhotoInsightError::from_message(format!("Failed to create temp file: {}", e))
        })?;
        temp_file.write_all(&image_data).map_err(|e| {
            PhotoInsightError::from_message(format!("Failed to write image data: {}", e))
        })?;

        // Get the file name as a string
        let temp_file_name = temp_path.to_string_lossy().to_string();
        let image =
            yolo_v8::image::Image::new(&temp_file_name, YoloV8ObjectDetection::input_dimension());
        let detections = yolo.predict(&image, 0.25, 0.7).postprocess().0;
        let result: Vec<DetectedObject> = detections
            .into_iter()
            .map(|bbox| DetectedObject {
                class_name: bbox.name.to_string(),
                confidence: bbox.conf as f32,
                bbox: (
                    bbox.xmin as f32,
                    bbox.ymin as f32,
                    bbox.xmax as f32,
                    bbox.ymax as f32,
                ),
            })
            .collect();
        results.push(AnalysisResult::new(photo_info, result));
    }
    Ok(results)
}
