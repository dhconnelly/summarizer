use axum::{
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

async fn index() -> Result<Html<String>, StatusCode> {
    let file = Assets::get("index.html").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let data = file.data.into_owned();
    let text = String::from_utf8(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(text))
}

async fn summarize() -> Result<Html<String>, StatusCode> {
    Ok(Html(String::from("<strong>API ERROR!</strong>")))
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(index))
        .route("/summarize", post(summarize));

    Ok(router.into())
}
