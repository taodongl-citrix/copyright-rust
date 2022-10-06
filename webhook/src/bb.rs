use crate::util;
use axum::extract::State;
use axum::{routing::post, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Ref {
    project: String,
    repository: String,
    branch: String,
    commit: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: u32,
    from: Ref,
    to: Ref,
}

#[derive(Debug, Clone)]
pub struct Bitbucket {
    username: String,
    password: String,
}

impl Bitbucket {
    pub fn new(u: &str, p: &str) -> Bitbucket {
        Bitbucket {
            username: u.to_string(),
            password: p.to_string(),
        }
    }
}

async fn bitbucket_event_handler(
    State(bitbucket): State<Bitbucket>,
    Json(event): Json<Event>,
) -> &'static str {
    let checked = util::protect_enter(
        "bitbucket",
        &event.to.project,
        &event.to.repository,
        event.id,
    );
    if !checked {
        tracing::warn!("the same request is running");
        return "duplicated request";
    }
    tokio::task::spawn(async move {
        util::run_command(
            &bitbucket.username,
            &bitbucket.password,
            event.id,
            &event.to.project,
            &event.to.repository,
            "bitbucket",
        )
        .await
    });
    "ok"
}

pub fn create(u: &str, p: &str) -> axum::Router<Bitbucket> {
    let bitbucket = Bitbucket::new(u, p);
    axum::Router::with_state(bitbucket).route("/hook", post(bitbucket_event_handler))
}
