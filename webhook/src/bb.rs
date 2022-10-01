use std::borrow::Borrow;
use crate::util;
use axum::{routing::post, extract::FromRequest, http::{StatusCode, Request}, async_trait, response::IntoResponse};
use axum::body::Body;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use tracing_subscriber::fmt::format;

#[derive(Debug, Clone)]
pub struct Bitbucket {
    username: String,
    password: String,
    secret: Vec<u8>,
}

impl Bitbucket {
    pub fn new(u: &str, p: &str, s: &[u8]) -> Bitbucket {
        Bitbucket {
            username: u.to_string(),
            password: p.to_string(),
            secret: s.to_vec()
        }
    }
}

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

static BITBUCKET_OPT: OnceCell<Bitbucket> = OnceCell::const_new();

#[async_trait]
impl<S> FromRequest<S, Body> for Event
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: Request<Body>, _: &S) -> Result<Self, Self::Rejection> {
        let (header, b) = req.into_parts();
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
        let secret = &BITBUCKET_OPT.get().unwrap().secret;
        let mut hmac = Hmac::<Sha256>::new_from_slice(secret).map_err(|err| (StatusCode::FORBIDDEN, err.to_string()).into_response())?;
        hmac.update(body.borrow());
        let verify = format!("{:x}", hmac.finalize().into_bytes());
        if verify != signature {
            return Err((StatusCode::FORBIDDEN, "signature verification failed".to_string()).into_response());
        }
        let ev: Event = serde_json::from_slice(body.borrow()).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;
        return Ok(ev);
    }
}

async fn bitbucket_event_handler(event: Event) -> Result<String, (StatusCode, String)> {
    let checked = util::GLOBAL_BARREL.try_put(event.id, &event.to.project, &event.to.repository, &event.to.branch);
    if let Err(x) = checked  {
        return Err((StatusCode::ACCEPTED, x.to_string()));
    }
    let username = &BITBUCKET_OPT.get().unwrap().username;
    let password = &BITBUCKET_OPT.get().unwrap().password;
    tokio::task::spawn(async move {
        util::run_command(username,
                          password,
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

pub fn create(u: &str, p: &str, s: &[u8]) -> axum::Router {
    BITBUCKET_OPT.set(Bitbucket::new(u, p, s)).unwrap();
    // let mut bb = BITBUCKET_OPT.write().unwrap();
    // bb.secret = s.as_bytes().to_vec();
    // bb.username = u.to_string();
    // bb.password = p.to_string();
    axum::Router::new()
        .route("/hook", post(bitbucket_event_handler))
}