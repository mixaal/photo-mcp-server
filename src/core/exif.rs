use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Read, path::Path};

use lazy_static::lazy_static;

use crate::core::{error::PhotoInsightError, image_cache::PhotoInfo, zip::is_image_file};

lazy_static! {
    static ref RE: Regex = Regex::new(r"^.?(\d\d\d\d)-(\d\d)").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExifInfo {
    pub year: u32,
    pub month: u32,
    pub model: String,
    pub width: u32,
    pub height: u32,
    pub date_time: String,
    pub aperture: String,
    pub shutter_speed: String,
    pub iso: String,
    pub focal_len: String,
    pub lens: String,
}

// Enum to represent different types of EXIF tag values
enum ExifTagValue {
    String(String),
    Number(u32),
    Float(f32),
}

impl ExifInfo {
    /// Checks if the EXIF information matches the given query parameters.
    pub fn matches_query(
        &self,
        tag_name: &String,
        tag_value: &String,
        operator: &String,
    ) -> Result<bool, PhotoInsightError> {
        let exif_tag_value = self.extract_tag_value(tag_name.as_str())?;
        ExifInfo::match_exif_tag_value(exif_tag_value, tag_value.as_str(), operator.as_str())
    }

    // Function to compare an ExifTagValue with a given tag value and operator (type aware)
    fn match_exif_tag_value(
        value: ExifTagValue,
        tag_value: &str,
        operator: &str,
    ) -> Result<bool, PhotoInsightError> {
        match value {
            ExifTagValue::String(s) => match operator {
                "==" => Ok(s.to_lowercase() == tag_value.to_lowercase()),
                "!=" => Ok(s.to_lowercase() != tag_value.to_lowercase()),
                "contains" => Ok(s.to_lowercase().contains(&tag_value.to_lowercase())),
                "starts_with" => Ok(s.to_lowercase().starts_with(&tag_value.to_lowercase())),
                "ends_with" => Ok(s.to_lowercase().ends_with(&tag_value.to_lowercase())),
                _ => Err(PhotoInsightError::from_message(format!(
                    "Invalid operator for string: {}",
                    operator
                ))),
            },
            ExifTagValue::Number(n) => {
                let tag_value: u32 = tag_value.parse().map_err(|_| {
                    PhotoInsightError::from_message("Invalid number value for comparison")
                })?;
                match operator {
                    "==" => Ok(n == tag_value),
                    "!=" => Ok(n != tag_value),
                    ">" => Ok(n > tag_value),
                    "<" => Ok(n < tag_value),
                    ">=" => Ok(n >= tag_value),
                    "<=" => Ok(n <= tag_value),
                    _ => Err(PhotoInsightError::from_message(format!(
                        "Invalid operator for number: {}",
                        operator
                    ))),
                }
            }
            ExifTagValue::Float(f) => {
                let tag_value: f32 = tag_value.parse().map_err(|_| {
                    PhotoInsightError::from_message("Invalid float value for comparison")
                })?;
                match operator {
                    "==" => Ok((f - tag_value).abs() < std::f32::EPSILON),
                    "!=" => Ok((f - tag_value).abs() >= std::f32::EPSILON),
                    ">" => Ok(f > tag_value),
                    "<" => Ok(f < tag_value),
                    ">=" => Ok(f >= tag_value),
                    "<=" => Ok(f <= tag_value),
                    _ => Err(PhotoInsightError::from_message(format!(
                        "Invalid operator for float: {}",
                        operator,
                    ))),
                }
            }
        }
    }

    // Extracts the value of a specified EXIF tag and returns it as an ExifTagValue enum
    fn extract_tag_value(&self, tag_name: &str) -> Result<ExifTagValue, PhotoInsightError> {
        match tag_name {
            "model" | "lens" => match tag_name {
                "model" => Ok(ExifTagValue::String(self.model.clone())),
                "lens" => Ok(ExifTagValue::String(self.lens.clone())),
                _ => Err(PhotoInsightError::from_message("Invalid tag name")),
            },
            "aperture" | "shutter_speed" | "iso" | "focal_len" => {
                let val = match tag_name {
                    "aperture" => &self.aperture,
                    "shutter_speed" => &self.shutter_speed,
                    "iso" => &self.iso,
                    "focal_len" => &self.focal_len,
                    _ => "",
                };
                let f: f32 = val
                    .parse()
                    .map_err(|_| PhotoInsightError::from_message("Invalid float value"))?;
                Ok(ExifTagValue::Float(f))
            }
            "width" | "height" | "year" | "month" => {
                let val = match tag_name {
                    "width" => self.width,
                    "height" => self.height,
                    "year" => self.year,
                    "month" => self.month,
                    _ => 0,
                };
                Ok(ExifTagValue::Number(val))
            }
            _ => Err(PhotoInsightError::from_message(format!(
                "Invalid tag name: {}",
                tag_name
            ))),
        }
    }
}

pub fn extract_all_exifs_from_zip_archive(
    image_dir: &str,
    zip_file_name: &str,
) -> Result<HashMap<PhotoInfo, ExifInfo>, PhotoInsightError> {
    let zip_path = Path::new(image_dir).join(zip_file_name);
    let mut files = HashMap::new();

    if zip_path.is_file() {
        let file = std::fs::File::open(&zip_path).map_err(|e| PhotoInsightError::new(e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| PhotoInsightError::new(e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| PhotoInsightError::new(e))?;
            let file_name = file.name().to_string();

            if is_image_file(&file_name) {
                let mut image_data = Vec::new();
                file.read_to_end(&mut image_data)
                    .map_err(|e| PhotoInsightError::new(e))?;
                let exif = extract_exif_info(&image_data, false);
                if exif.is_err() {
                    tracing::warn!(
                        "Failed to extract exif from image {} in zip {}: {}",
                        file_name,
                        zip_file_name,
                        exif.err().unwrap()
                    );
                    continue;
                }
                files.insert(
                    PhotoInfo::new(zip_file_name.to_owned(), file_name, i),
                    exif.unwrap().0,
                );
            }
        }
    } else {
        return Err(PhotoInsightError::from_message(
            "Provided zip file path is not a file",
        ));
    }
    Ok(files)
}

pub fn extract_exif_info(
    image_data: &Vec<u8>,
    thumbnail: bool,
) -> Result<(ExifInfo, Vec<u8>), PhotoInsightError> {
    let mut cursor = std::io::Cursor::new(image_data);
    let exifreader = exif::Reader::new();
    let exif = exifreader
        .read_from_container(&mut cursor)
        .map_err(|e| PhotoInsightError::new(e))?;

    let model = extract_tag(&exif, vec![exif::Tag::Model], false);

    let w = extract_tag(
        &exif,
        vec![exif::Tag::ImageWidth, exif::Tag::PixelXDimension],
        true,
    );
    let h = extract_tag(
        &exif,
        vec![exif::Tag::ImageLength, exif::Tag::PixelYDimension],
        true,
    );
    let width: u32 = w
        .parse()
        .map_err(|_| PhotoInsightError::from_message("invalid width"))?;
    let height: u32 = h
        .parse()
        .map_err(|_| PhotoInsightError::from_message("invalid height"))?;

    let date_time = extract_tag(
        &exif,
        vec![
            exif::Tag::DateTimeOriginal,
            exif::Tag::DateTime,
            exif::Tag::DateTimeDigitized,
        ],
        false,
    );

    let (year, month) = if let Some(caps) = RE.captures(&date_time) {
        (
            caps[1]
                .parse::<u32>()
                .map_err(|_| PhotoInsightError::from_message("invalid year"))?,
            caps[2]
                .parse::<u32>()
                .map_err(|_| PhotoInsightError::from_message("invalid month"))?,
        )
    } else {
        (0, 0)
    };

    let aperture = extract_tag(
        &exif,
        vec![exif::Tag::FNumber, exif::Tag::ApertureValue],
        true,
    );
    let mut shutter = extract_tag(
        &exif,
        vec![exif::Tag::ShutterSpeedValue, exif::Tag::ExposureTime],
        false,
    );
    if !shutter.contains("/") {
        shutter = extract_tag(&exif, vec![exif::Tag::ExposureTime], false);
    }
    if shutter.contains("/") {
        // This might look lame, but the lines above tried to extract the value which contains '/'
        let s = shutter.split("/").collect::<Vec<&str>>();
        if s.len() == 2 {
            shutter = String::from(s[1]);
        }
    }
    shutter = shutter.replace("\"", ""); // trim double-quotes if present
    let shutter_speed = shutter; //.parse::<f32>().unwrap_or_default();
    let iso = extract_tag(
        &exif,
        vec![exif::Tag::ISOSpeed, exif::Tag::PhotographicSensitivity],
        true,
    );
    let focal_len = extract_tag(
        &exif,
        vec![exif::Tag::FocalLengthIn35mmFilm, exif::Tag::FocalLength],
        true,
    );
    let lens = extract_tag(
        &exif,
        vec![
            exif::Tag::LensModel,
            exif::Tag::LensSpecification,
            exif::Tag::LensMake,
        ],
        false,
    );

    let maker_notes = extract_tag(&exif, vec![exif::Tag::MakerNote], false);
    println!("maker_notes={maker_notes}");

    // println!("model={}", model.replace("\"", "").replace(",", ""));

    Ok((
        ExifInfo {
            year,
            month,
            model,
            width,
            height,
            date_time,
            aperture,
            shutter_speed,
            iso,
            focal_len,
            lens,
        },
        if thumbnail {
            extract_thm(image_data, &exif)
        } else {
            Vec::new()
        },
    ))
}

fn extract_thm(image_data: &Vec<u8>, exif: &exif::Exif) -> Vec<u8> {
    //let buf = fs::read(path).expect("read input file");
    let buf = exif.buf();
    let off = exif
        .get_field(exif::Tag::JPEGInterchangeFormat, exif::In::THUMBNAIL)
        .and_then(|f| f.value.get_uint(0));
    let len = exif
        .get_field(exif::Tag::JPEGInterchangeFormatLength, exif::In::THUMBNAIL)
        .and_then(|f| f.value.get_uint(0));

    if off.is_some() && len.is_some() {
        // have thumbnail
        // println!("XXX: {}, {}", off.unwrap(), len.unwrap());
        let start = off.unwrap() as usize;
        let end = start + len.unwrap() as usize;
        let res = &buf[start..end];
        // println!("start={} end={}", start, end);
        res.to_vec()
    } else {
        // fallback to canvas resize if we are unable to extract the thumbnail from the exif tags
        let w = extract_tag(
            &exif,
            vec![exif::Tag::ImageWidth, exif::Tag::PixelXDimension],
            true,
        );
        let h = extract_tag(
            &exif,
            vec![exif::Tag::ImageLength, exif::Tag::PixelYDimension],
            true,
        );

        let orig_w: u32 = w.parse().unwrap();
        let orig_h: u32 = h.parse().unwrap();

        resize(image_data, orig_w, orig_h)
    }
}

fn extract_tag(exif: &exif::Exif, tags: Vec<exif::Tag>, numeric: bool) -> String {
    for t in tags.iter() {
        let v = exif.get_field(*t, exif::In::PRIMARY);
        if v.is_some() {
            let mut val = v.unwrap().display_value().to_string();
            if !numeric {
                val = val.replace("\"", "").replace(",", "");
                val = String::from(val.trim());
                // if !val.ends_with("\"") {
                val = val + "\"";
                // }
                // if !val.starts_with("\"") {
                val = String::from("\"") + &val;
                // }
            } else {
                // in case of numeric requirement if we get string tag that is present but
                // non-parsable into number, we return "0"
                let Ok(_) = val.parse::<f32>() else {
                    return String::from("0");
                };
            }
            return val;
        }
    }
    return String::from(if numeric { "0" } else { "\"unknown\"" });
}

pub(crate) fn resize(buf: &Vec<u8>, orig_w: u32, orig_h: u32) -> Vec<u8> {
    // load the image
    let img = image::load_from_memory(&buf).expect("image decoded");

    let width = if orig_w == 0 { img.width() } else { orig_w };
    let height = if orig_h == 0 { img.height() } else { orig_h };

    let mut nw: u32 = 160;
    let mut nh: u32 = 100;
    if height > width {
        nw = 100;
        nh = 160;
    }
    tracing::info!("Resizing image {width}x{height} -> {nw}x{nh}");
    let sc_img = img.resize(nw, nh, image::imageops::FilterType::Lanczos3);
    // sc_img.as_bytes().to_vec()
    sc_img.save("/tmp/x.jpg").expect("resize save failed");
    let result = std::fs::read("/tmp/x.jpg").expect("read resized file");
    result
}

#[cfg(test)]
mod tests {
    use crate::core::exif::extract_exif_info;

    #[test]
    fn test_exif_info() {
        let img = std::fs::read("test/images/40d.jpg").expect("image not found");
        let exif = extract_exif_info(&img, false).expect("can't extract exif");
        println!("{exif:#?}");
    }
}
