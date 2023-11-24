use axum;
use summarizer;
use tokio;
use tracing;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    let key = std::env::var("OPENAI_API_KEY").expect("missing OPENAI_API_KEY");
    let port = std::env::var("PORT")
        .map(|port| port.parse())
        .unwrap_or(Ok(8080))
        .expect("invalid PORT");
    let router = summarizer::make_router(key);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    tracing_subscriber::fmt().with_target(false).compact().init();
    tracing::info!("starting server at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("failed to start server");
}
