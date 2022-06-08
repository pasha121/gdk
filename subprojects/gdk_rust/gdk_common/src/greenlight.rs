use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInfo {
    pub cert: String,    // ascii
    pub key: String,     // ascii
    pub node_id: String, // hex
}
