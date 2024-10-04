use super::*;

#[derive(Debug, Deserialize, Clone, Serialize,PartialEq, Eq)]
pub struct AttributeString {
    #[serde(rename = "maxLength")]
    pub(crate) max_length: Option<u32>,

    #[serde(rename = "minLength")]
    pub(crate) min_length: Option<u32>,
}
