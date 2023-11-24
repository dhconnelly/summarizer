use axum;
use summarizer;
use tokio;

#[tokio::main]
async fn main() {
    let key = std::env::var("OPENAI_API_KEY").expect("missing OPENAI_API_KEY");
    let port = std::env::var("PORT").unwrap_or(String::from("8080"));
    let port = port.parse::<u16>().unwrap();
    let router = summarizer::make_router(key);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
