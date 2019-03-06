
use std::borrow::Cow;
use std::io::Write;
use std::str::FromStr;

use shell_escape::escape;

use crate::errors::{AppError, AppResultU, AppResult};
use crate::meta::Meta;



pub enum OutputFormat {
    Chrysoberyl,
    Simple,
}

impl OutputFormat {
    pub fn write<W: Write>(&self, w: &mut W, meta: &Meta) -> AppResultU {
        use OutputFormat::*;

        match self {
            Simple =>
                writeln!(w, "{}", meta.file.path)?,
            Chrysoberyl => {
                write!(w, "@push-image")?;
                write!(w, " --meta width={}", meta.dimensions.width)?;
                write!(w, " --meta height={}", meta.dimensions.width)?;
                write!(w, " --meta format={}", meta.format)?;
                write!(w, " --meta dhash={}", meta.dhash)?;
                writeln!(w, " {}", escape(Cow::from(&meta.file.path)))?;
            },
        }
        Ok(())
    }
}

impl FromStr for OutputFormat {
    type Err = AppError;

    fn from_str(s: &str) -> AppResult<Self> {
        use OutputFormat::*;

        let result = match s {
            "c" | "chrysoberyl" => Chrysoberyl,
            "s" | "simple" => Simple,
            _ => return Err(AppError::InvalidOutputFormat(s.to_owned())),
        };
        Ok(result)
    }
}
