use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct AttributeInteger {
    pub min: Option<u32>,
    pub max: Option<u32>,
}
