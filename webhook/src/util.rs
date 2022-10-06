use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::OnceCell;
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::RwLock;

static GLOBAL_DATA: RwLock<Vec<u64>> = RwLock::new(Vec::new());

pub fn protect_enter(scm: &str, project: &str, repository: &str, id: u32) -> bool {
    let key = get_key(scm, id, project, repository);
    {
        let found = GLOBAL_DATA.read().unwrap().iter().any(|x| *x == key);
        // tracing::info!("key is {}, found: {}", key, found);
        if found {
            return false;
        }
    }
    {
        GLOBAL_DATA.write().unwrap().push(key);
        // tracing::info!("add key is {}", key);
    }
    true
}

pub fn protect_leave(scm: &str, project: &str, repository: &str, id: u32) {
    let key = get_key(scm, id, project, repository);
    let mut item = GLOBAL_DATA.write().unwrap();
    let index = item.iter().position(|x| *x == key);
    if let Some(idx) = index {
        item.remove(idx);
    }
}

fn get_key(scm: &str, id: u32, project: &str, repository: &str) -> u64 {
    let key = format!("{}-{}-{}-{}", scm, id, project, repository);
    let mut s = DefaultHasher::new();
    s.write(&key.as_bytes());
    s.finish()
}

static WORK_DIR: OnceCell<PathBuf> = OnceCell::const_new();

async fn get_work_dir() -> PathBuf {
    let path = std::env::current_exe().unwrap();
    let folder = path.parent().unwrap();
    let data = folder.join("data");
    if !data.is_dir() {
        let ret = tokio::fs::create_dir(data).await;
        if let Err(e) = ret {
            tracing::error!("{:?}", e);
        }
    }
    folder.to_path_buf()
}

pub async fn run_command(
    username: &str,
    password: &str,
    id: u32,
    project: &str,
    repository: &str,
    scm: &str,
) {
    let dir = WORK_DIR.get_or_init(get_work_dir).await;
    
    let mut command = tokio::process::Command::new(
        dir.join("work"),
    );
    command
        .env("GIT_USERNAME", username)
        .env("GIT_PASSWORD", password)
        .env(
            "GIT_ASKPASS",
            dir.join("askpass.sh"),
        )
        .args([
            format!("--project={}", project),
            format!("--repository={}", repository),
            format!("--id={}", id),
            format!("--scm={}", scm),
        ])
        .current_dir(dir.join("data"));
    let mut proc = command.spawn().unwrap();
    let ret = proc.wait().await;
    match ret {
        Ok(_) => tracing::debug!(
            "project: {}, repository: {}, id: {} is completed",
            project,
            repository,
            id
        ),
        Err(e) => tracing::debug!(
            "project: {}, repository: {}, id: {} ran with error {:?}",
            project,
            repository,
            id,
            e
        ),
    }
    protect_leave(scm, project, repository, id);
}

fn check_signature(
    signature: &str,
    data: &[u8],
    secret: &[u8],
) -> Result<(), (StatusCode, String)> {
    let mut hmac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|err| (StatusCode::FORBIDDEN, err.to_string()))?;
    hmac.update(data);
    let verify = format!("{:x}", hmac.finalize().into_bytes());
    if verify != signature {
        return Err((
            StatusCode::FORBIDDEN,
            "signature isn't verified".to_string(),
        ));
    }
    Ok(())
}

pub async fn signature_middleware(
    State(secret): State<Vec<u8>>,
    req: Request<hyper::Body>,
    next: Next<hyper::Body>,
) -> Result<impl IntoResponse, Response> {
    if secret.is_empty() {
        return Ok(next.run(req).await);
    }
    let (parts, body) = req.into_parts();
    let bytes = hyper::body::to_bytes(body)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;
    let hdr = parts
        .headers
        .get("X-Hub-Signature-256")
        .ok_or((StatusCode::FORBIDDEN, "'X-Hub-Signature-256': not found").into_response())?;
    let message = hdr
        .to_str()
        .map_err(|e| (StatusCode::FORBIDDEN, e.to_string()).into_response())?;
    let signature = message
        .strip_prefix("sha256=")
        .ok_or((StatusCode::FORBIDDEN, "signature is wrong format").into_response())?;
    check_signature(&signature.to_lowercase(), bytes.borrow(), &secret)
        .map_err(|e| e.into_response())?;
    let request = Request::from_parts(parts, hyper::Body::from(bytes));
    Ok(next.run(request).await)
}
