use crate::store::Value;
use std::collections::HashMap;

pub mod decode;

#[derive(Default)]
pub struct RDB {
    pub header: String,
    pub metadata: HashMap<String, String>,
    pub database: HashMap<String, Value>,
    pub expiry_time: HashMap<String, std::time::Instant>,
}
