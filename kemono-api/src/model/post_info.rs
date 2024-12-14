use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PostInfo {
    pub post: Value,
    pub attachments: Vec<AttachmentLike>,
    pub previews: Vec<AttachmentLike>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentLike {
    pub server: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
}
