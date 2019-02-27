
use std::path::Path;

use chrono::DateTime;
use chrono::offset::Utc;
use immeta;
use serde_derive::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult, from_os_str};



#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub animation: bool,
    pub dimensions: Dimensions,
    pub mime_type: &'static str,
    pub file: FileMeta,
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
    pub fn from_file<T: AsRef<Path>>(file: &T) -> AppResult<Meta> {
        let file_meta = std::fs::metadata(file)?;
        let file_meta = FileMeta {
            path: from_os_str(file.as_ref().as_os_str())?.to_string(),
            size: file_meta.len() as u32,
            created: file_meta.created().ok().map(DateTime::from),
            modified: file_meta.modified().ok().map(DateTime::from),
            accessed: file_meta.accessed().ok().map(DateTime::from),
        };
        from_file(file, file_meta).map_err(|err| {
            AppError::ImageLoading(err, format!("{:?}", file.as_ref()))
        })
    }
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: dim={}x{} type={} anim={}",
            self.file.path,
            self.dimensions.width,
            self.dimensions.height,
            self.mime_type,
            self.animation)
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

fn from_file<T: AsRef<Path>>(file: &T, file_meta: FileMeta) -> Result<Meta, immeta::Error> {
    use immeta::GenericMetadata::*;

    const IMAGE_PREFIX: &str = "image/";

    let meta = immeta::load_from_file(file)?;
    let dimensions = meta.dimensions();

    let animation = match meta {
        Gif(ref meta) => meta.is_animated(),
        _ => false,
    };

    let mut mime_type = meta.mime_type();
    if mime_type.starts_with(IMAGE_PREFIX) {
        mime_type = &mime_type[IMAGE_PREFIX.len() ..];
    }

    let meta = Meta {
        animation,
        dimensions: Dimensions {
            height: dimensions.height,
            width: dimensions.width,
        },
        file: file_meta,
        mime_type,
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
