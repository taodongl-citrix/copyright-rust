mod bb;
mod util;
mod gh;
use tokio::io::AsyncReadExt;
use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "webhook=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let mut file = tokio::fs::File::open("/Users/taodongl/Downloads/copyright-app.pem").await.unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.unwrap();
    let ss = "0143207be7c417eb8444a552d78b61deffa64efd".as_bytes();
    let bb = bb::create("username", "name", &ss);
    let gh = gh::create("243479", &buffer, &ss);
    let app = Router::new()
        .nest("/api/bb", bb)
        .nest("/api/gh", gh)
        .route("/api/ping", get(|| async {"pong"}));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}