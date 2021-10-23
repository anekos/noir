
use std::convert::From;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use chrono::DateTime;
use chrono::offset::Utc;
use image::GenericImageView;
use serde_derive::{Deserialize, Serialize};

use crate::errors::{AppResult, from_os_str};



#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub animation: bool,
    pub dhash: Option<String>,
    pub dimensions: Dimensions,
    pub file: FileMeta,
    pub format: &'static str,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dimensions {
    pub height: u32,
    pub width: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileMeta {
    pub path: String,
    pub size: u32,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub accessed: Option<DateTime<Utc>>,
}


impl Meta {
    pub fn from_file<T: AsRef<Path>>(file: &T, compute_dhash: bool) -> AppResult<Meta> {
        let file_meta = std::fs::metadata(file)?;
        let file_meta = FileMeta {
            path: from_os_str(file.as_ref().as_os_str())?.to_string(),
            size: file_meta.len() as u32,
            created: file_meta.created().ok().map(DateTime::from),
            modified: file_meta.modified().ok().map(DateTime::from),
            accessed: file_meta.accessed().ok().map(DateTime::from),
        };
        from_file(file, file_meta, compute_dhash)
    }
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: dim={}x{} format={} anim={}",
            self.file.path,
            self.dimensions.width,
            self.dimensions.height,
            self.format,
            self.animation)?;
        if let Some(ref dhash) = &self.dhash {
            write!(f, " dhash={}", dhash)?;
        }
        Ok(())
    }
}


impl From<&image_meta::Dimensions> for Dimensions {
    fn from(meta: &image_meta::Dimensions) -> Self {
        Self { width: meta.width, height: meta.height }
    }
}

impl Dimensions {
    pub fn ratio(&self) -> (u32, u32) {
        let divisor = gcd(self.width, self.height);
        if divisor == 0 {
            (0, 0)
        } else {
            (self.width / divisor, self.height / divisor)
        }
    }
}

fn from_file<T: AsRef<Path>>(file: &T, file_meta: FileMeta, hashing: bool) -> AppResult<Meta> {
    if hashing {
        from_file_with_hashing(file, file_meta)
    } else {
        from_file_without_hashing(file, file_meta)
    }
}

fn from_file_without_hashing<T: AsRef<Path>>(file: &T, file_meta: FileMeta) -> AppResult<Meta> {
    use crate::image_format::ImageFormatExt;

    let meta = image_meta::load_from_file(file)?;
    let animation = meta.is_animation();
    let format = meta.to_str();
    let dimensions = &meta.dimensions;

    let meta = Meta {
        animation,
        dhash: None,
        dimensions: Dimensions {
            height: dimensions.height,
            width: dimensions.width,
        },
        file: file_meta,
        format,
    };

    Ok(meta)
}

fn from_file_with_hashing<T: AsRef<Path>>(file: &T, file_meta: FileMeta) -> AppResult<Meta> {
    use crate::image_format::ImageFormatExt;

    let meta = image_meta::load_from_file(file)?;

    let mut file = File::open(file)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;
    let format = image::guess_format(&content)?;
    let image = image::load_from_memory_with_format(&content, format)?;
    let dhash = Some(format!("{:016x}" ,dhash::get_dhash(&image)));

    let meta = Meta {
        animation: meta.is_animation(),
        dhash,
        dimensions: Dimensions {
            height: image.height(),
            width: image.width(),
        },
        file: file_meta,
        format: format.to_str(),
    };

    Ok(meta)
}

fn gcd(x: u32, y: u32) -> u32 {
    if y == 0 {
        x
    } else {
        gcd(y, x % y)
    }
}
