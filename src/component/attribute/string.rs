use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct AttributeString {
    #[serde(rename = "maxLength")]
    pub(crate) max_length: Option<u32>,

    #[serde(rename = "minLength")]
    pub(crate) min_length: Option<u32>,
}
