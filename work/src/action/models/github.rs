use serde_json::Value;
use serde::Serialize;
use serde::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubPayload {
    #[serde(default)]
    pub data: Value,
    pub errors: Option<Vec<GithubErrorPayload>>,
}

// Add comment
pub const GITHUB_ADD_COMMENT: &'static str = r#"mutation {
  addComment(input: {subjectId: "{{id}}", body: "{{body}}", clientMutationId: "copyright-add-comment"}) {
    clientMutationId
  }
}
"#;

// Delete comment
pub const GITHUB_DELETE_COMMENT: &'static str = r#"mutation {
  deleteIssueComment(input: {id: "{{id}}", clientMutationId: "copyright-delete-comment"}) {
    clientMutationId
  }
}
"#;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubDeleteCommentPayload {
    pub delete_issue_comment: GithubDeleteIssueComment,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubDeleteIssueComment {
    pub client_mutation_id: Value,
}

// Query Files in Pull-Request
pub const GITHUB_QUERY: &'static str = r#"query {
  repository(owner: "{{project}}", name: "{{repository}}") {
    pullRequest(number: {{number}}) {
      id
      files(first: 50{% if file_after != "" %},  after: "{{file_after}}" {% endif %}) {
        edges {
          node {
            path
          }
        }
        pageInfo {
          endCursor
          hasNextPage
        }
      }
      comments(first: 50{% if comment_after != "" %}, after: "{{comment_after}}" {% endif %}) {
        edges {
          node {
            id
            body
          }
        }
        pageInfo {
          endCursor
          hasNextPage
        }
      }
    }
  }
}
"#;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubPullRequestPayload {
    pub repository: GithubRepository,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubRepository {
    pub pull_request: GithubPullRequest,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubPullRequest {
    pub id: String,
    pub files: GithubFilesInPull,
    pub comments: GithubCommentsInPull,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubFilesInPull {
    pub edges: Vec<GithubFilesEdge>,
    pub page_info: PageInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubFilesEdge {
    pub node: GithubFilesNode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubFilesNode {
    pub path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCommentsInPull {
    pub edges: Vec<GithubCommentsEdge>,
    pub page_info: PageInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCommentsEdge {
    pub node: GithubCommentsNode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCommentsNode {
    pub id: String,
    pub body: String,
}

// Error
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubErrorPayload {
    pub message: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct GithubErrorLocation {
//     pub line: i64,
//     pub column: i64,
// }