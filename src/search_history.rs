
use serde_derive::{Deserialize, Serialize};



#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SearchHistory {
    pub expression: String,
    pub uses: i64,
}
