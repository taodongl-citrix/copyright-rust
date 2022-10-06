use crate::util;
use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::{
    http::{Request, StatusCode},
    routing::post,
    Json, RequestExt,
};
use serde::{Deserialize, Serialize};
use std::time;

#[derive(Debug, Serialize, Deserialize)]
struct AccessToken {
    token: String,
    expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubAccount {
    login: String,
    id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubAppInstallation {
    id: u32,
    access_tokens_url: String,
    account: GithubAccount,
    app_id: u32,
    target_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Owner {
    login: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repo {
    name: String,
    full_name: String,
    owner: Owner,
}

#[derive(Debug, Serialize, Deserialize)]
struct Branch {
    sha: String,
    #[serde(alias = "ref")]
    name: String,
    repo: Repo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Installation {
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubPullRequest {
    head: Branch,
    base: Branch,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubPayload {
    action: String,
    number: u32,
    pull_request: GithubPullRequest,
    installation: Installation,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    /// The time that this JWT was issued
    iat: u64,
    // JWT expiration time
    exp: u64,
    // GitHub App's identifier number
    iss: String,
}

impl JwtClaims {
    fn new(app_id: &str) -> JwtClaims {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        JwtClaims {
            // The time that this JWT was issued (now)
            iat: now - 10,
            // JWT expiration time (8 minute from now)
            exp: now + 60 * 8,
            // GitHub App's identifier number
            iss: app_id.to_string(),
        }
    }
}

fn get_id_token(app_id: &str, key: &[u8]) -> String {
    let claims = JwtClaims::new(app_id);
    let header = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::RS256,
        ..Default::default()
    };
    let private_key = jsonwebtoken::EncodingKey::from_rsa_pem(key).unwrap();
    let token = jsonwebtoken::encode(&header, &claims, &private_key);
    return token.unwrap();
}

async fn github_event_handler(
    client: State<reqwest::Client>,
    req: Request<Body>,
) -> Result<&'static str, (StatusCode, String)> {
    let event_type = req
        .headers()
        .get("X-GitHub-Event")
        .ok_or((StatusCode::BAD_REQUEST, "event type is missing".to_string()))?;
    if HeaderValue::from_static("pull_request") != event_type {
        return Err((StatusCode::BAD_REQUEST, "event type is wrong".to_string()));
    }
    let Json(payload): Json<GithubPayload> = req.extract().await.map_err(internal_error)?;
    if payload.action != "synchronize" && payload.action != "opened" {
        return Err((StatusCode::BAD_REQUEST, "action type is wrong".to_string()));
    }
    let repository = payload.pull_request.base.repo.name;
    let owner = payload.pull_request.base.repo.owner.login;
    let yes = util::protect_enter("github", &owner, &repository, payload.number);
    if !yes {
        tracing::warn!("the same request is running");
        return Ok("duplicated request");
    }
    let response = client
        .post(format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            payload.installation.id
        ))
        .send()
        .await
        .map_err(internal_error)?;
    let access: AccessToken = response.json().await.map_err(internal_error)?;
    tokio::task::spawn(async move {
        util::run_command(
            "x-access-token",
            &access.token,
            payload.number,
            &owner,
            &repository,
            "github",
        )
        .await
    });
    Ok("ok")
}

pub fn create(id: &str, key: &[u8]) -> axum::Router<reqwest::Client> {
    let token = get_id_token(id, key);
    let bearer = format!("Bearer {}", token);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&bearer).unwrap(),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .user_agent("copyright-webhook/0.1.0")
        .build()
        .unwrap();
    axum::Router::with_state(client).route("/hook", post(github_event_handler))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
