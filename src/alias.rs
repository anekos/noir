
use serde_derive::{Deserialize, Serialize};



#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Alias {
    pub expression: String,
    pub recursive: bool,
}
