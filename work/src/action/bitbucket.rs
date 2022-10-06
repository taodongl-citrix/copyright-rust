use reqwest::blocking::Client;
use serde_json::json;
// use serde::{Serialize, Deserialize};
use crate::action::models::{
    BitbucketActivitiesPayload, BitbucketChangesPayload, BitbucketPagePayload,
};
use crate::action::{git_fetch, scan};
use crate::Handler;

use super::models::{BAD_COMMENT, GOOD_COMMENT};

pub struct Bitbucket {
    pub project: String,
    pub repository: String,
    pub id: u32,
    client: Client,
    base_url: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct Comment {
//     text: String,
// }

impl Bitbucket {
    pub fn new(project: &str, repository: &str, id: u32) -> Bitbucket {
        let username = std::env::var("GIT_USERNAME").expect("GIT_USERNAME is not set");
        let password = std::env::var("GIT_PASSWORD").expect("GIT_PASSWORD is not set");
        let auth = format!("{}:{}", username, password);
        let mut header = reqwest::header::HeaderMap::new();
        let basic_auth = format!("Basic {}", base64::encode(&auth));
        header.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&basic_auth).unwrap(),
        );
        header.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        header.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::blocking::ClientBuilder::new()
            .default_headers(header)
            .build()
            .unwrap();
        Bitbucket {
            project: project.to_string(),
            repository: repository.to_string(),
            id,
            client: client.to_owned(),
            base_url: format!("https://code-dev.do.citrite.net/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}", &project, &repository, id),
        }
    }

    fn get_changed_files(&self) -> anyhow::Result<Vec<String>> {
        let mut start = 0;
        let mut files: Vec<String> = vec![];
        loop {
            let url = format!("{}/changes?start={}", &self.base_url, start);
            let resp = self.client.get(&url).send()?;
            if resp.status().is_success() {
                let payload: BitbucketPagePayload = resp.json()?;
                let changes: Vec<BitbucketChangesPayload> = serde_json::from_value(payload.values)?;
                for value in changes.iter() {
                    files.push(String::from(&value.path.to_string));
                }
                if payload.is_last_page {
                    return Ok(files);
                }
                if let Some(next) = payload.next_page_start {
                    start = next;
                } else {
                    return Ok(files);
                }
            } else {
                return Err(anyhow::anyhow!(resp.status().to_string()));
            }
        }
    }

    fn get_comment(&self) -> anyhow::Result<Option<(i32, i32)>> {
        let mut start = 0;
        loop {
            let url = format!("{}/activities?start={}", self.base_url, start);
            let resp = self.client.get(&url).send()?;
            if resp.status().is_success() {
                let payload: BitbucketPagePayload = resp.json()?;
                let activities: Vec<BitbucketActivitiesPayload> =
                    serde_json::from_value(payload.values)?;
                for value in activities.iter() {
                    if let Some(comment) = &value.comment {
                        if comment.text.ends_with("reported by CICD") {
                            return Ok(Some((comment.id, comment.version)));
                        }
                    }
                }
                if payload.is_last_page {
                    break;
                }
                if let Some(next) = payload.next_page_start {
                    start = next;
                } else {
                    break;
                }
            } else {
                return Err(anyhow::anyhow!(resp.status().to_string()));
            }
        }
        Ok(None)
    }

    fn delete_comment(&self, id: i32, version: i32) -> anyhow::Result<()> {
        let url = format!(
            "{baseUrl}/comments/{id}?version={version}",
            baseUrl = self.base_url,
            id = id,
            version = version
        );
        let resp = self.client.delete(&url).send()?;
        if resp.status().is_success() {
            Ok(())
        } else {
            return Err(anyhow::anyhow!(resp.status().to_string()));
        }
    }

    fn create_comment(&self, positive: bool) -> anyhow::Result<()> {
        let url = format!("{baseUrl}/comments", baseUrl = self.base_url);
        let message = if positive {
            BAD_COMMENT
        } else {
            GOOD_COMMENT
        };
        let body = json!({ "text": message });
        //let resp = self.client.post(&url).json(&Comment{text: message.to_string()}).send()?;
        let resp = self.client.post(&url).json(&body).send()?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(resp.status().to_string()))
        }
    }
}

impl Handler for Bitbucket {
    fn execute(&mut self) -> anyhow::Result<()> {
        let files = self.get_changed_files()?;
        git_fetch(
            &files,
            &format!(
                "https://code-dev.do.citrite.net/scm/{project}/{repo}.git",
                project = self.project,
                repo = self.repository
            ),
            self.id,
        )?;
        let yes = scan()?;
        let comment_opt = self.get_comment()?;
        if let Some(comment) = comment_opt {
            self.delete_comment(comment.0, comment.1)?;
        }
        tracing::info!("create pull-request for scanned result: {}", yes);
        self.create_comment(yes)?;
        Ok(())
    }
}
