
use std::ffi::OsStr;

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
    #[fail(display = "{}: {}", 0, 1)]
    ImageLoading(immeta::Error, String),
    #[fail(display = "IO error: {}", 0)]
    Io(std::io::Error),
    #[fail(display = "YAML Error: {}", 0)]
    Serde(serde_yaml::Error),
    #[fail(display = "Database error: {}", 0)]
    Sqlite(rusqlite::Error),
    #[fail(display = "UTF-8 error")]
    Utf8,
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
define_error!(rusqlite::Error, Sqlite);
define_error!(serde_yaml::Error, Serde);
define_error!(std::io::Error, Io);
define_error!(walkdir::Error, DirectoryWalking);

pub fn from_os_str(s: &OsStr) -> AppResult<&str> {
    s.to_str().ok_or(AppError::Utf8)
}
