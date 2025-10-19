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
    pub(crate) photo_info: PhotoInfo,
    pub(crate) object_detection: Vec<DetectedObject>,
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
    use yolo_v8::YoloV8ObjectDetection;

    let yolo = YoloV8ObjectDetection::new().map_err(|e| PhotoInsightError::new(e))?;

    let mut results = Vec::new();
    for (photo_info, image_data) in images {
        let image = yolo_v8::image::Image::load_from_memory(
            &image_data,
            YoloV8ObjectDetection::input_dimension(),
        )
        .map_err(|e| PhotoInsightError::new(e))?;
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
