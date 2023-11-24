use async_openai::{config::OpenAIConfig, types, Client};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use lazy_static::{initialize, lazy_static};
use rust_embed::RustEmbed;
use serde::Deserialize;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

lazy_static! {
    static ref INDEX: Html<String> = {
        let data = Assets::get("index.html").unwrap().data.into_owned();
        let text = String::from_utf8(data).unwrap();
        Html(text)
    };
}

async fn index() -> Html<String> {
    INDEX.clone()
}

#[derive(Clone)]
pub struct AppState {
    pub client: Client<OpenAIConfig>,
}

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
}

fn make_request(text: &str) -> types::CreateChatCompletionRequest {
    let msg = types::ChatCompletionRequestUserMessageArgs::default()
        .content(format!("Summarize the following text:\n\n{}", text))
        .build()
        .unwrap()
        .into();
    types::CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([msg])
        .build()
        .unwrap()
}

fn extract_response(
    response: types::CreateChatCompletionResponse,
) -> Result<String, &'static str> {
    response
        .choices
        .into_iter()
        .next()
        .ok_or("no choices in response")?
        .message
        .content
        .ok_or("no content in message")
}

fn openai_error<S: ToString>(msg: S) -> StatusCode {
    tracing::error!("openai api: {}", msg.to_string());
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn summarize(
    State(AppState { client }): State<AppState>,
    Form(form): Form<SummarizeRequest>,
) -> Result<Html<String>, StatusCode> {
    let request = make_request(&form.text);
    let response = client.chat().create(request).await.map_err(openai_error)?;
    let content = extract_response(response).map_err(openai_error)?;
    Ok(Html(content))
}

pub fn make_router(key: String) -> Router {
    initialize(&INDEX);
    let client = Client::with_config(OpenAIConfig::new().with_api_key(key));
    let state = AppState { client };
    let tracer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));
    Router::new()
        .route("/", get(index))
        .route("/summarize", post(summarize))
        .with_state(state)
        .layer(tracer)
}
