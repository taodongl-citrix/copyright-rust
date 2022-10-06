mod bb;
mod gh;
mod util;
use axum::routing::get;
use axum::{middleware, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "webhook=debug");
    }
    tracing_subscriber::fmt::init();

    let secret = std::env::var("SECRET_KEY").expect("SECRET_KEY");
    let bitbucket_username =
        std::env::var("BITBUCKET_USERNAME").expect("BITBUCKET_USERNAME");
    let bitbucket_password =
        std::env::var("BITBUCKET_PASSWORD").expect("BITBUCKET_PASSWORD");
    let git_app_id = std::env::var("GITHUB_APPID").expect("GITHUB_APPID");
    let git_app_key = std::env::var("GITHUB_APPKEY").expect("GITHUB_APPKEY");

    let bb = bb::create(&bitbucket_username, &bitbucket_password);
    let gh = gh::create(&git_app_id, git_app_key.as_bytes());

    let api = Router::new()
        .nest("/bb", bb)
        .nest("/gh", gh)
        .layer(middleware::from_fn_with_state(
            secret.as_bytes().to_vec(),
            util::signature_middleware,
        ));
    let app = Router::new()
        .nest("/api", api)
        .route("/ping", get(|| async { "pong" }));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
