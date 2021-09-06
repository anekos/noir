
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use actix_web::{
    error, dev::HttpResponseBuilder, http::header, http::StatusCode, HttpResponse,
};
use failure::Fail;



pub type AppResult<T> = Result<T, AppError>;
pub type AppResultU = Result<(), AppError>;



#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "Application directory error: {}", 0)]
    AppDir(app_dirs::AppDirsError),
    #[fail(display = "clap: {}", 0)]
    Clap(clap::Error),
    #[fail(display = "Failed to load directory: {}", 0)]
    DirectoryWalking(walkdir::Error),
    #[fail(display = "{}", 0)]
    Format(std::fmt::Error),
    #[fail(display = "{}", 0)]
    FromSql(rusqlite::types::FromSqlError),
    #[fail(display = "Invalid number format")]
    InvalidNumberFormat(std::num::ParseIntError),
    #[fail(display = "{}", 0)]
    ImageLoading(image::ImageError),
    #[fail(display = "{}", 0)]
    ImageMetaLoading(image_meta::ImageError),
    #[fail(display = "Invalid output format name: {}", 0)]
    InvalidOutputFormat(String),
    #[fail(display = "Invalid tag format: {}", 0)]
    InvalidTagFormat(String),
    #[fail(display = "IO error: {}", 0)]
    Io(std::io::Error),
    #[fail(display = "JSON Error: {}", 0)]
    SerdeJson(serde_json::Error),
    #[fail(display = "YAML Error: {}", 0)]
    SerdeYaml(serde_yaml::Error),
    #[fail(display = "Database error: {}", 0)]
    Sqlite(rusqlite::Error),
    #[fail(display = "Tag generator failed: {}", 0)]
    TagGeneratorFailed(String),
    #[fail(display = "UTF-8 error")]
    UnknownUtf8,
    #[fail(display = "UTF-8 error: {}", 0)]
    Utf8(std::string::FromUtf8Error),
    #[fail(display = "")]
    Void,
    #[fail(display = "{} for {:?}", 0, 1)]
    WithPath(Box<AppError>, PathBuf),
}


macro_rules! define_error {
    ($source:ty, $kind:ident) => {
        impl From<$source> for AppError {
            fn from(error: $source) -> AppError {
                AppError::$kind(error)
            }
        }
    }
}

define_error!(app_dirs::AppDirsError, AppDir);
define_error!(clap::Error, Clap);
define_error!(image::ImageError, ImageLoading);
define_error!(image_meta::ImageError, ImageMetaLoading);
define_error!(rusqlite::Error, Sqlite);
define_error!(rusqlite::types::FromSqlError, FromSql);
define_error!(serde_json::Error, SerdeJson);
define_error!(serde_yaml::Error, SerdeYaml);
define_error!(std::fmt::Error, Format);
define_error!(std::num::ParseIntError, InvalidNumberFormat);
define_error!(std::string::FromUtf8Error, Utf8);
define_error!(walkdir::Error, DirectoryWalking);

pub fn from_os_str(s: &OsStr) -> AppResult<&str> {
    s.to_str().ok_or(AppError::UnknownUtf8)
}

pub fn from_path<T: AsRef<Path>>(path: &T) -> AppResult<&str> {
    from_os_str(path.as_ref().as_os_str())
}

pub fn wrap_with_path<T: AsRef<Path>, U>(path: &T, result: AppResult<U>) -> AppResult<U> {
    result.map_err(|it| match it {
        AppError::WithPath(_, _) => it,
        _ => AppError::WithPath(Box::new(it), path.as_ref().to_path_buf()),
    })
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> AppError {
        if error.kind() == std::io::ErrorKind::BrokenPipe {
            AppError::Void
        } else {
            AppError::Io(error)
        }
    }
}


impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
