
use serde_derive::{Deserialize, Serialize};



#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Alias {
    pub expression: String,
    pub recursive: bool,
}
