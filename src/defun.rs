use std::sync::Arc;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, Error, Result};
use wildmatch::WildMatch;


type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub fn add_match_functions(db: &Connection) -> Result<()> {
    add_match_function(db, "match", false)?;
    add_match_function(db, "imatch", true)?;
    Ok(())
}

fn add_match_function(db: &Connection, name: &'static str, ignore_case: bool) -> Result<()> {
    // https://docs.rs/rusqlite/0.26.0/rusqlite/functions/index.html
    db.create_scalar_function(
        name,
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");
            let m: Arc<WildMatch> = ctx.get_or_create_aux(0, |vr| -> Result<_, BoxError> {
                if ignore_case {
                    Ok(WildMatch::new(&vr.as_str()?.to_lowercase()))
                } else {
                    Ok(WildMatch::new(vr.as_str()?))
                }
            })?;
            let is_match = {
                let text = ctx
                    .get_raw(1)
                    .as_str()
                    .map_err(|e| Error::UserFunctionError(e.into()))?;
                if ignore_case {
                    m.matches(&text.to_lowercase())
                } else {
                    m.matches(text)
                }

            };

            Ok(is_match)
        },
    )
}
