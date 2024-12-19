use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PostInfo {
    pub post: Post,
    pub attachments: Vec<AttachmentLike>,
    pub previews: Vec<AttachmentLike>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentLike {
    pub server: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub user: String,
    pub service: String,
    pub title: String,
    pub content: String,
    pub embed: Embed,
    pub shared_file: bool,
    pub added: String,
    pub published: String,
    pub edited: Option<String>,
    pub file: File,
    pub attachments: Vec<AttachmentLike>,
    pub poll: Option<Poll>,
    pub captions: Option<String>,
    pub tags: Option<Vec<String>>,
    pub next: Option<String>,
    pub prev: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embed {
    pub url: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Poll {
    pub title: String,
    pub choices: Vec<Choice>,
    pub closes_at: Option<String>,
    pub created_at: String,
    pub description: Option<String>,
    pub allows_multiple: bool,
    pub total_votes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub text: String,
    pub votes: i64,
}
