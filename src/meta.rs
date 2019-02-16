
use std::path::Path;

use immeta;

use crate::errors::{AppError, AppResult};



#[derive(Clone, Debug)]
pub struct Meta {
    pub animation: bool,
    pub dimensions: Dimensions,
    pub file_size: u64,
    pub mime_type: &'static str,
}

#[derive(Clone, Debug)]
pub struct Dimensions {
    pub height: u32,
    pub width: u32,
}



impl Meta {
    pub fn from_file<T: AsRef<Path>>(file: &T) -> AppResult<Meta> {
        from_file(file).map_err(|err| {
            AppError::ImageLoading(err, format!("{:?}", file.as_ref()))
        })
    }
}


impl Dimensions {
    pub fn ratio(&self) -> (u32, u32) {
        let divisor = gcd(self.width, self.height);
        (self.width / divisor, self.height / divisor)
    }
}

fn from_file<T: AsRef<Path>>(file: &T) -> Result<Meta, immeta::Error> {
    use immeta::GenericMetadata::*;

    const IMAGE_PREFIX: &str = "image/";

    let file_size = std::fs::metadata(file)?.len();

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
        file_size,
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
