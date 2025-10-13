use serde::{Deserialize, Serialize};

use crate::{
    IC,
    core::{error::PhotoInsightError, exif, traversal, yolo::AnalysisResult, zip},
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhotoInfo {
    /// Zip file name in the filesystem
    pub zip_file_name: String,
    /// Image file name inside the zip file
    pub photo_file_name: String,
    /// Image index inside the zip file, useful for extraction
    pub photo_index_in_zip: usize,
}

impl PhotoInfo {
    pub fn new(zip_file: String, image: String, index: usize) -> Self {
        PhotoInfo {
            zip_file_name: zip_file,
            photo_file_name: image,
            photo_index_in_zip: index,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExifResult {
    file: PhotoInfo,
    exif: exif::ExifInfo,
}

impl ExifResult {
    fn new(file: PhotoInfo, exif: exif::ExifInfo) -> Self {
        Self { file, exif }
    }
}

// year => month => zip_file => image_file => exif_info
pub type ByYearMonth = HashMap<u32, HashMap<u32, Vec<PhotoInfo>>>;

// zip_file => image_file => exif_info
pub type ExifCache = HashMap<PhotoInfo, exif::ExifInfo>;
pub type ExifCacheSerialized = HashMap<String, exif::ExifInfo>;

pub struct PhotoCache {
    image_dir: String,
    // Map image file name to zip file name
    pub images: Vec<PhotoInfo>,
    pub exif_cache: ExifCache,
    pub by_year_month: ByYearMonth,
}

impl PhotoCache {
    pub fn build(image_dir: &str) -> Result<Self, PhotoInsightError> {
        let mut exif_cache: ExifCache = HashMap::new();
        let mut by_year_month: ByYearMonth = HashMap::new();
        let mut zip_infos = HashSet::new();
        let zip_files = traversal::list_directory_zip_files(image_dir)?;
        for zip in &zip_files {
            let images = zip::list_zip_archive(image_dir, zip)?;
            tracing::info!("Found zip file: {} with {} images", zip, images.len());
            for (index, image) in &images {
                zip_infos.insert(PhotoInfo::new(zip.clone(), image.clone(), *index));
            }

            // Extract and cache exif data
            if !std::path::Path::new(&form_file(image_dir, zip, "exif")).exists() {
                tracing::info!(
                    "Exif file does not exists for zip {}, creating  exif data",
                    zip
                );

                let extract_exif_raw: HashMap<PhotoInfo, exif::ExifInfo> =
                    crate::core::exif::extract_all_exifs_from_zip_archive(image_dir, zip)?;
                let exif_count = extract_exif_raw.len();
                tracing::info!("Extracted exif from {} images in zip {}", exif_count, zip);

                // Convert ZipInfo to String for serialization
                let extract_exif: ExifCacheSerialized = extract_exif_raw
                    .into_iter()
                    .map(|(zip_info, exif)| {
                        (
                            format!(
                                "{}|{}|{}",
                                zip_info.zip_file_name,
                                zip_info.photo_file_name,
                                zip_info.photo_index_in_zip
                            ),
                            exif,
                        )
                    })
                    .collect();

                serde_json::to_writer_pretty(
                    std::fs::File::create(form_file(image_dir, zip, "exif"))
                        .map_err(|e| PhotoInsightError::new(e))?,
                    &extract_exif,
                )
                .map_err(|e| PhotoInsightError::new(e))?;
            } else {
                tracing::info!(
                    "Exif file already exists for zip {}, skipping exif extraction",
                    zip
                );
            }
            let extract_exif_serialized: ExifCacheSerialized = serde_json::from_reader(
                std::fs::File::open(form_file(image_dir, zip, "exif"))
                    .map_err(|e| PhotoInsightError::new(e))?,
            )
            .map_err(|e| PhotoInsightError::new(e))?;

            // Convert String back to ZipInfo
            let extract_exif: ExifCache = extract_exif_serialized
                .into_iter()
                .filter_map(|(key, exif)| {
                    let parts: Vec<&str> = key.split('|').collect();
                    if parts.len() == 3 {
                        let zip_file = parts[0].to_string();
                        let image = parts[1].to_string();
                        let index = parts[2].parse::<usize>().ok()?;
                        Some((PhotoInfo::new(zip_file, image, index), exif))
                    } else {
                        None
                    }
                })
                .collect();

            // merge extract_exif into exif_cache
            exif_cache.extend(extract_exif.clone());

            // Extract and cache by year month data
            if !std::path::Path::new(&form_file(image_dir, zip, "by_year_month")).exists() {
                tracing::info!(
                    "By year month file does not exists for zip {}, creating by year month data",
                    zip
                );
                let by_year_month: ByYearMonth =
                    extract_exif
                        .iter()
                        .fold(HashMap::new(), |mut acc, (zip_info, exif)| {
                            let year = exif.year;
                            let month = exif.month;
                            acc.entry(year)
                                .or_insert_with(HashMap::new)
                                .entry(month)
                                .or_insert_with(Vec::new)
                                .push(zip_info.clone());
                            acc
                        });
                serde_json::to_writer_pretty(
                    std::fs::File::create(form_file(image_dir, zip, "by_year_month"))
                        .map_err(|e| PhotoInsightError::new(e))?,
                    &by_year_month,
                )
                .map_err(|e| PhotoInsightError::new(e))?;
            } else {
                tracing::info!(
                    "By year month file already exists for zip {}, skipping by year month creation",
                    zip
                );
            }
            let partial_by_year_month: ByYearMonth = serde_json::from_reader(
                std::fs::File::open(form_file(image_dir, zip, "by_year_month"))
                    .map_err(|e| PhotoInsightError::new(e))?,
            )
            .map_err(|e| PhotoInsightError::new(e))?;

            // merge partial_by_year_month into by_year_month
            for (year, month_map) in partial_by_year_month {
                let mut updates: Vec<(u32, u32, Vec<PhotoInfo>)> = Vec::new();
                for (month, infos) in month_map {
                    updates.push((year, month, infos));
                }
                for (year, month, infos) in updates {
                    by_year_month
                        .entry(year)
                        .or_insert_with(HashMap::new)
                        .entry(month)
                        .or_insert_with(Vec::new)
                        .extend(infos);
                }
            }
        }
        Ok(Self {
            images: zip_infos.into_iter().collect(),
            image_dir: image_dir.to_string(),
            exif_cache,
            by_year_month,
        })
    }

    // List all images in the cache
    pub fn list_all_images(&self, offset: usize, limit: usize) -> (Vec<&PhotoInfo>, usize) {
        let total_images = self.images.len();
        tracing::info!("Total images in cache: {}", total_images);
        let start = offset.min(total_images);
        let end = (offset + limit).min(total_images);
        tracing::info!("Returning images from {} to {}", start, end);
        (self.images[start..end].iter().collect(), total_images)
    }

    // Search for image by partial name (case insensitive)
    // returns vector exif info and thumbnail image data
    pub fn search_image_by_name(
        &self,
        image_name: &str,
        offset: usize,
        limit: usize,
    ) -> (Vec<&PhotoInfo>, usize) {
        let image_name_lower = image_name.to_lowercase();
        let zip_infos: Vec<&PhotoInfo> = self
            .images
            .iter()
            .filter(|info| {
                info.photo_file_name
                    .to_lowercase()
                    .contains(&image_name_lower)
            })
            .collect();
        let total_found = zip_infos.len();
        tracing::info!("Found {} matching images", total_found);
        let start = offset.min(zip_infos.len());
        let end = (offset + limit).min(zip_infos.len());
        tracing::info!("Returning images from {} to {}", start, end);

        (zip_infos[start..end].to_vec(), total_found)
    }

    pub fn search_image_by_year_month(
        &self,
        year: u32,
        month: u32,
        offset: usize,
        limit: usize,
    ) -> (Vec<&PhotoInfo>, usize) {
        let r = IC.by_year_month.get(&year);
        if r.is_none() {
            return (Vec::new(), 0);
        }
        let month_map = r.unwrap();
        let r = month_map.get(&month);
        if r.is_none() {
            return (Vec::new(), 0);
        }

        let zip_infos: &Vec<PhotoInfo> = r.unwrap();
        let total_found = zip_infos.len();
        tracing::info!("Found {} matching images", total_found);
        let start = offset.min(zip_infos.len());
        let end = (offset + limit).min(zip_infos.len());
        tracing::info!("Returning images from {} to {}", start, end);

        let slice = zip_infos[start..end].iter().collect::<Vec<&PhotoInfo>>();

        (slice, total_found)
    }

    pub fn search_image_by_exif_tags(
        &self,
        tag_name: &String,
        tag_value: &String,
        operator: &String,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<ExifResult>, usize), PhotoInsightError> {
        tracing::info!("search image by EXIF tag : offset: {offset} Limiting results to {limit}");
        let mut results = Vec::new();
        IC.exif_cache.iter().for_each(|(zip_info, exif)| {
            let matched = exif
                .matches_query(tag_name, tag_value, operator)
                .map_err(|e| e)
                .unwrap_or(false);

            if matched {
                results.push(ExifResult::new(zip_info.clone(), exif.clone()));
            }
        });

        let total_found = results.len();
        tracing::info!("Found {} matching images", total_found);
        let start = offset.min(results.len());
        let end = (offset + limit).min(results.len());
        tracing::info!("Returning images from {} to {}", start, end);

        let slice = results[start..end].to_vec();

        Ok((slice, total_found))
    }

    pub fn exif_info(
        &self,
        image_infos: Vec<&PhotoInfo>,
    ) -> Result<Vec<ExifResult>, PhotoInsightError> {
        let mut exif_infos = Vec::new();
        for img in image_infos {
            if let Some(exif) = self.exif_cache.get(img) {
                exif_infos.push(ExifResult::new(img.clone(), exif.clone()));
            }
        }
        Ok(exif_infos)
    }

    pub fn image_data(
        &self,
        image_infos: Vec<&PhotoInfo>,
    ) -> Result<Vec<(PhotoInfo, String, Vec<u8>)>, PhotoInsightError> {
        let mut arxives = HashMap::new();
        for info in image_infos {
            let arxive = info.zip_file_name.clone();
            let index = info.photo_index_in_zip;
            arxives.entry(arxive).or_insert_with(Vec::new).push(index);
        }
        let mut images = Vec::new();
        for (zip_file, indices) in arxives {
            let unpacked = zip::extract_zip_archive(&self.image_dir, &zip_file, indices)?;
            for (photo_info, image_data) in unpacked {
                let exif = crate::core::exif::extract_exif_info(&image_data, true);
                if exif.is_err() {
                    tracing::warn!(
                        "Failed to extract exif from image {:?} in zip {}: {}",
                        photo_info,
                        zip_file,
                        exif.err().unwrap()
                    );
                    // let mime = mime_from_image(&image_data);
                    let resized_image = exif::resize(&image_data, 0, 0);
                    let mime = mime_from_image(&resized_image);
                    images.push((photo_info, mime, resized_image));
                } else {
                    let image_data = exif.unwrap().1;
                    let mime = mime_from_image(&image_data);
                    images.push((photo_info, mime, image_data));
                }
            }
        }
        Ok(images)
    }

    pub fn yolo_v8_analysis(
        &self,
        image_infos: Vec<&PhotoInfo>,
    ) -> Result<Vec<AnalysisResult>, PhotoInsightError> {
        let mut arxives = HashMap::new();
        for info in image_infos {
            let arxive = info.zip_file_name.clone();
            let index = info.photo_index_in_zip;
            arxives.entry(arxive).or_insert_with(Vec::new).push(index);
        }
        let mut analysis_results = Vec::new();
        for (zip_file, indices) in arxives {
            let unpacked = zip::extract_zip_archive(&self.image_dir, &zip_file, indices)?;
            let yolo_results = crate::core::yolo::analyze_images_using_yolo(unpacked)?;
            analysis_results.extend(yolo_results);
        }
        Ok(analysis_results)
    }
}

fn form_file(image_dir: &str, zip_file: &str, suffix: &str) -> String {
    format!("{}/{}.{}.json", image_dir, zip_file, suffix)
}

fn mime_from_image(image_data: &Vec<u8>) -> String {
    match crate::core::image::guess_format(image_data) {
        Ok(format) => match format {
            crate::core::image::ImageFormat::Png => "image/png;base64".to_string(),
            crate::core::image::ImageFormat::Jpeg => "image/jpeg;base64".to_string(),
            crate::core::image::ImageFormat::Gif => "image/gif;base64".to_string(),
            crate::core::image::ImageFormat::Bmp => "image/bmp;base64".to_string(),
            crate::core::image::ImageFormat::Tiff => "image/tiff;base64".to_string(),
            crate::core::image::ImageFormat::WebP => "image/webp;base64".to_string(),
            crate::core::image::ImageFormat::Pnm => "image/pnm;base64".to_string(),
            crate::core::image::ImageFormat::Tga => "image/tga;base64".to_string(),
            crate::core::image::ImageFormat::Dds => "image/dds;base64".to_string(),
            crate::core::image::ImageFormat::Ico => "image/ico;base64".to_string(),
            crate::core::image::ImageFormat::Hdr => "image/hdr;base64".to_string(),
            crate::core::image::ImageFormat::OpenExr => "image/openexr;base64".to_string(),
            crate::core::image::ImageFormat::Farbfeld => "image/farbfeld;base64".to_string(),
            crate::core::image::ImageFormat::Avif => "image/avif;base64".to_string(),
            crate::core::image::ImageFormat::Qoi => "image/qoi;base64".to_string(),
            crate::core::image::ImageFormat::Pcx => "image/pcx;base64".to_string(),
        },
        Err(_) => "application/octet-stream;base64".to_string(),
    }
    // let ext = std::path::Path::new(file_name)
    //     .extension()
    //     .and_then(std::ffi::OsStr::to_str)
    //     .unwrap_or("")
    //     .to_lowercase();
    // match ext.to_lowercase().as_str() {
    //     "jpg" | "jpeg" => "image/jpeg;base64".to_string(),
    //     "png" => "image/png;base64".to_string(),
    //     "gif" => "image/gif;base64".to_string(),
    //     "bmp" => "image/bmp;base64".to_string(),
    //     "tiff" => "image/tiff;base64".to_string(),
    //     "webp" => "image/webp;base64".to_string(),
    //     _ => "application/octet-stream;base64".to_string(),
    // }
}
