use serde::Serialize;
use serde::Deserialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketPagePayload {
    pub size: u32,
    pub limit: u32,
    pub is_last_page: bool,
    pub values: Value,
    pub next_page_start: Option<u32>,
    pub start: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketChangesPayload {
    pub content_id: Option<String>,
    pub from_content_id: Option<String>,
    pub path: Path,
    pub executable: Option<bool>,
    pub percent_unchanged: Option<i64>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub node_type: Option<String>,
    pub src_path: Option<SrcPath>,
    pub src_executable: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Path {
    pub components: Vec<String>,
    pub parent: String,
    pub name: String,
    pub extension: String,
    pub to_string: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SrcPath {
    pub components: Vec<String>,
    pub parent: String,
    pub name: String,
    pub extension: String,
    pub to_string: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketActivitiesPayload {
    pub id: i64,
    pub created_date: i64,
    pub user: BitbucketUser,
    pub action: String,
    pub comment_action: Option<String>,
    pub comment: Option<Comment>,
    pub comment_anchor: Option<CommentAnchor>,
    pub from_hash: Option<String>,
    pub previous_from_hash: Option<String>,
    pub previous_to_hash: Option<String>,
    pub to_hash: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketUser {
    pub name: String,
    pub email_address: String,
    pub id: i64,
    pub display_name: String,
    pub active: bool,
    pub slug: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    // pub properties: Properties,
    pub id: i32,
    pub version: i32,
    pub text: String,
    pub author: BitbucketUser,
    pub created_date: i64,
    pub updated_date: i64,
    pub severity: String,
    pub state: String,
    //pub permitted_operations: PermittedOperations2,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Properties {
//     pub key: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Author {
//     pub name: String,
//     pub email_address: String,
//     pub id: i64,
//     pub display_name: String,
//     pub active: bool,
//     pub slug: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Properties2 {
//     pub key: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PermittedOperations2 {
//     pub editable: bool,
//     pub deletable: bool,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentAnchor {
    pub line: i64,
    pub line_type: String,
    pub file_type: String,
    pub path: String,
    pub src_path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Added {
    pub commits: Vec<Commit>,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub id: String,
    pub display_id: String,
    pub author: BitbucketUser,
    pub author_timestamp: i64,
    pub committer: Committer,
    pub committer_timestamp: i64,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Committer {
    pub name: String,
    pub email_address: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Removed {
//     pub commits: Vec<Commit>,
//     pub total: i64,
// }