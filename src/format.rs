
use image::ImageFormat;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};



pub trait FormatExt {
    fn to_str(&self) -> &'static str;
}


impl FormatExt for ImageFormat {
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
