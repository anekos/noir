
use crate::errors::AppResult;
use crate::meta::Meta;
use crate::database::Database;



pub fn select(db: &Database, expression: &str) -> AppResult<Vec<Meta>> {
    let result = db.select(expression)?;
    Ok(result)
}



