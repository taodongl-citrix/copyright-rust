use std::borrow::Borrow;
use std::time;
use axum::{routing::post, extract::FromRequest, http::{StatusCode, Request}, async_trait, response::IntoResponse};
use axum::extract::State;
use hmac::{Hmac, Mac};
use axum::body::Body;
use axum::http::HeaderValue;
use tokio::sync::OnceCell;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use crate::util;

#[derive(Debug, Serialize)]
struct Github {
    secret: Vec<u8>,
}

impl Github {
    fn new(s: &[u8]) -> Github {
        Github {
            secret: s.to_vec()
        }
    }
}
static GITHUB_OPT: OnceCell<Github> = OnceCell::const_new();

#[derive(Debug, Serialize, Deserialize)]
struct Ref {
    project: String,
    repository: String,
    branch: String,
    commit: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: u32,
    from: Ref,
    to: Ref
}

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
struct Repo {
    name: String,
    full_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Branch {
    sha: String,
    #[serde(alias = "ref")]
    name: String,
    repo: Repo,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubPullRequest {
    head: Branch,
    base: Branch,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubEvent {
    action: String,
    number: u32,
    pull_request: GithubPullRequest,
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
            .duration_since(time::UNIX_EPOCH).unwrap()
            .as_secs();
        JwtClaims {
            // The time that this JWT was issued (now)
            iat: now - 10,
            // JWT expiration time (8 minute from now)
            exp: now + 60*8,
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
    let private_key =
        jsonwebtoken::EncodingKey::from_rsa_pem(key).unwrap();
    let token = jsonwebtoken::encode(&header, &claims, &private_key);
    return token.unwrap();
}

#[async_trait]
impl<S> FromRequest<S, Body> for Event
    where
        S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: Request<Body>, _: &S) -> Result<Self, Self::Rejection> {
        let (header, b) = req.into_parts();
        let event_type = header.headers.get("X-GitHub-Event");
        let mut checked = false;
        if let Some(tye) = event_type {
            if HeaderValue::from_static("pull_request") == tye {
                checked = true;
            }
        }
        if !checked {
            return Err((StatusCode::BAD_REQUEST, "event type is wrong".to_string()).into_response());
        }
        let body = hyper::body::to_bytes(b)
            .await.map_err(|err| (StatusCode::FORBIDDEN, err.to_string()).into_response())?;
        let hdr = header.headers.get("X-Hub-Signature-256");
        let signature = match hdr {
            Some(x) => {
                let s = x.to_str().unwrap();
                s.strip_prefix("sha256=").unwrap().to_lowercase()
            },
            None => return Err((StatusCode::FORBIDDEN, "signature missing".to_string()).into_response())
        };
        let secret = &GITHUB_OPT.get().unwrap().secret;
        let mut hmac = Hmac::<Sha256>::new_from_slice(secret).map_err(|err| (StatusCode::FORBIDDEN, err.to_string()).into_response())?;
        hmac.update(body.borrow());
        let verify = format!("{:x}", hmac.finalize().into_bytes());
        if verify != signature {
            return Err((StatusCode::FORBIDDEN, "signature verification failed".to_string()).into_response());
        }
        let ev: GithubEvent = serde_json::from_slice(body.borrow()).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;
        return Ok(Event {
            id: ev.number,
            from: Ref {
                project: "".to_string(),
                repository: ev.pull_request.head.repo.name,
                branch: ev.pull_request.head.name,
                commit: ev.pull_request.head.sha,
            },
            to: Ref {
                project: "".to_string(),
                repository: ev.pull_request.base.repo.name,
                branch: ev.pull_request.base.name,
                commit: ev.pull_request.base.sha,
            }
        });
    }
}


async fn github_event_handler(client: State<reqwest::Client>, event: Event) -> Result<String, (StatusCode, String)> {
    let checked = util::GLOBAL_BARREL.try_put(event.id, &event.to.project, &event.to.repository, &event.to.branch);
    if let Err(x) = checked  {
        return Err((StatusCode::ACCEPTED, x.to_string()));
    }
    let target = event.from.project.to_lowercase();
    let r0 = client.get("https://api.github.com/app/installations")
        .send()
        .await.map_err(internal_error)?;
    let installs: Vec<GithubAppInstallation> = r0.json().await.map_err(internal_error)?;
    let installation_opt = installs.iter().find(|x| (*x).account.login.to_lowercase() == target);
    let installation = match installation_opt {
        Some(x) => x,
        None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "app id not installed".to_string()))
    };
    let r1 = client.post(format!("https://api.github.com/app/installations/{}/access_tokens", installation.id))
        .send()
        .await.map_err(internal_error)?;
    let access: AccessToken = r1.json().await.map_err(internal_error)?;
    tokio::task::spawn(async move {
        util::run_command("x-access-token",
                          &access.token,
                          event.id,
                          &event.to.project,
                          &event.to.repository,
                          &event.to.branch,
                          &event.to.commit,
                          &event.from.project,
                          &event.from.repository,
                          &event.from.branch,
                          &event.from.commit,
        ).await
    });
    Ok("ok".to_string())
}

pub fn create(id: &str, key: &[u8], secret: &[u8]) -> axum::Router<reqwest::Client> {
    let token = get_id_token(id, key);
    let bearer = format!("Bearer {}", token);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, HeaderValue::from_str(&bearer).unwrap());
    headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("application/vnd.github+json"));
    let client = reqwest::Client::builder().default_headers(headers).build().unwrap();
    GITHUB_OPT.set(Github::new(secret)).unwrap();
    axum::Router::with_state(client)
        .route("/hook", post(github_event_handler))
}
fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
