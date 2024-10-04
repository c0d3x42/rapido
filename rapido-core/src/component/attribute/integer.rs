use super::*;

#[derive(Debug, Deserialize, Clone, Serialize,PartialEq, Eq)]
pub struct AttributeInteger {
    pub min: Option<u32>,
    pub max: Option<u32>,
}
