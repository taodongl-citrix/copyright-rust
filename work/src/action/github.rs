use crate::action::models::{
    GithubPayload, GithubPullRequestPayload, GITHUB_ADD_COMMENT, GITHUB_DELETE_COMMENT,
    GITHUB_QUERY, BAD_COMMENT,
};
use crate::action::{git_fetch, scan, Handler};
use reqwest::blocking::{Client, Response};
use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;

pub struct Github {
    project: String,
    repository: String,
    id: u32,
    client: Client,
}
#[derive(Debug, Serialize, Deserialize)]
struct GraphqlQuery {
    query: String,
}

#[derive(Debug)]
struct PullRequest {
    id: String,
    comment: Option<Comment>,
    files: Vec<String>,
}

#[derive(Debug)]
struct Comment {
    body: String,
    id: String,
}
impl Clone for Comment {
    fn clone(&self) -> Comment {
        Comment {
            id: self.id.clone(),
            body: self.body.clone(),
        }
    }
}

impl Github {
    pub fn new(project: &str, repository: &str, id: u32) -> Github {
        let token = std::env::var("GIT_PASSWORD").expect("GIT_PASSWORD is not set");
        let mut header = reqwest::header::HeaderMap::new();
        header.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        header.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        );
        header.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::blocking::ClientBuilder::new()
            .default_headers(header)
            .user_agent("copyright-rust/0.1.0")
            .build()
            .unwrap();
        Github {
            project: project.to_string(),
            repository: repository.to_string(),
            id,
            client: client.to_owned(),
        }
    }

    fn get_pull_request(&self) -> anyhow::Result<PullRequest> {
        let mut template = tera::Tera::default();
        let mut files: Vec<String> = Vec::new();
        let mut comments: Vec<Comment> = Vec::new();
        let mut file_after: String = "".to_string();
        let mut comment_after: String = "".to_string();
        let mut pull_request_node: String = "".to_string();
        loop {
            let mut context = tera::Context::new();
            context.insert("file_after", &file_after);
            context.insert("comment_after", &comment_after);
            context.insert("project", &self.project);
            context.insert("repository", &self.repository);
            context.insert("number", &format!("{}", self.id));
            let body = template.render_str(GITHUB_QUERY, &context)?;
            let resp: Response = self
                .client
                .post("https://api.github.com/graphql")
                .json(&GraphqlQuery { query: body })
                .send()
                .unwrap();
            if resp.status().is_success() {
                let payload: GithubPayload = resp.json().unwrap();
                if let Some(errors) = payload.errors {
                    return Err(anyhow::anyhow!(errors.first().unwrap().message.clone()));
                }
                let data: GithubPullRequestPayload = serde_json::from_value(payload.data).unwrap();
                if pull_request_node.is_empty() {
                    pull_request_node = data.repository.pull_request.id;
                }
                data.repository
                    .pull_request
                    .files
                    .edges
                    .iter()
                    .map(|x| x.node.path.deref())
                    .for_each(|x| files.push(x.to_string()));
                data.repository
                    .pull_request
                    .comments
                    .edges
                    .iter()
                    .map(|x| Comment {
                        id: x.node.id.to_string(),
                        body: x.node.body.to_string(),
                    })
                    .for_each(|x| comments.push(x));
                if !data.repository.pull_request.files.page_info.has_next_page
                    && !data
                        .repository
                        .pull_request
                        .comments
                        .page_info
                        .has_next_page
                {
                    break;
                }
                if let Some(next) = data.repository.pull_request.files.page_info.end_cursor {
                    file_after = next;
                }
                if let Some(next) = data.repository.pull_request.comments.page_info.end_cursor {
                    comment_after = next;
                }
            } else {
                return Err(anyhow::anyhow!(resp.status().to_string()));
            }
        }
        let comment_ptr = comments.iter().find(|x| x.body == BAD_COMMENT);
        let comment = match comment_ptr {
            Some(x) => Some(x.clone()),
            None => None,
        };
        Ok(PullRequest {
            id: pull_request_node,
            files,
            comment,
        })
    }
    fn delete_comment(&self, id: &str) -> anyhow::Result<()> {
        let mut template = tera::Tera::default();
        let mut context = tera::Context::new();
        context.insert("id", id);
        let body = template.render_str(GITHUB_DELETE_COMMENT, &context)?;
        let resp: Response = self
            .client
            .post("https://api.github.com/graphql")
            .json(&GraphqlQuery { query: body })
            .send()
            .unwrap();
        if resp.status().is_success() {
            let payload: GithubPayload = resp.json().unwrap();
            if let Some(errors) = payload.errors {
                return Err(anyhow::anyhow!(errors.first().unwrap().message.clone()));
            }
        } else {
            return Err(anyhow::anyhow!(resp.status().to_string()));
        }
        Ok(())
    }
    fn create_comment(&self, id: &str) -> anyhow::Result<()> {
        let mut template = tera::Tera::default();
        let mut context = tera::Context::new();
        context.insert("id", id);
        context.insert("body", BAD_COMMENT);
        let body = template.render_str(GITHUB_ADD_COMMENT, &context)?;
        let resp: Response = self
            .client
            .post("https://api.github.com/graphql")
            .json(&GraphqlQuery { query: body })
            .send()
            .unwrap();
        if resp.status().is_success() {
            let payload: GithubPayload = resp.json().unwrap();
            if let Some(errors) = payload.errors {
                return Err(anyhow::anyhow!(errors.first().unwrap().message.clone()));
            }
        } else {
            return Err(anyhow::anyhow!(resp.status().to_string()));
        }
        Ok(())
    }
}

impl Handler for Github {
    fn execute(&mut self) -> anyhow::Result<()> {
        let pull_request = self.get_pull_request()?;
        git_fetch(
            &pull_request.files,
            &format!(
                "https://github.com/{project}/{repo}.git",
                project = self.project,
                repo = self.repository
            ),
            self.id,
        )?;
        let yes = scan()?;
        if yes && pull_request.comment.is_none() {
            tracing::info!("report comment to pull-request");
            self.create_comment(&pull_request.id)?;
        } else if !yes && pull_request.comment.is_some() {
            tracing::info!("remove comment from pull-request");
            self.delete_comment(&pull_request.comment.unwrap().id)?;
        } else {
            tracing::info!("keep comment in pull-request");
        }
        Ok(())
    }
}
