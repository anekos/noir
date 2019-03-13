
use image::ImageFormat;
use image_meta::ImageMeta;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};



pub trait ImageFormatExt {
    fn to_str(&self) -> &'static str;
}


impl ImageFormatExt for ImageFormat {
    fn to_str(&self) -> &'static str {
        use ImageFormat::*;

        match self {
            BMP => "bmp",
            GIF => "gif",
            HDR => "hdr",
            ICO => "ico",
            JPEG => "jpeg",
            PNG => "png",
            PNM => "pnm", // portable-anymap-format
            TGA => "tga", // targa
            TIFF => "tiff",
            WEBP => "webp",
        }
    }
}

impl ImageFormatExt for ImageMeta {
    fn to_str(&self) -> &'static str {
        use image_meta::Format::*;

        match self.format {
            Gif => "gif",
            Jpeg => "jpeg",
            Png => "png",
        }
    }
}

fn from_str(s: &str) -> ImageFormat {
    use ImageFormat::*;
    match s {
        "bmp" => BMP,
        "gif" => GIF,
        "hdr" => HDR,
        "ico" => ICO,
        "jpeg" => JPEG,
        "png" => PNG,
        "pnm" => PNM,
        "tga" => TGA,
        "tiff" => TIFF,
        "webp" => WEBP,
        _ => panic!("Unknown format: {:?}", s),
    }
}

pub fn from_raw(value: ValueRef<'_>) -> FromSqlResult<ImageFormat> {
    String::column_result(value).map(|it| from_str(&it))
}
