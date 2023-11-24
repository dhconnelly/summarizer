use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use tracing::{instrument, Level};

#[derive(Clone)]
pub struct AppState {
    pub openai_api_key: String,
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
}

#[derive(Debug)]
struct Response(StatusCode, String);

impl Response {
    fn new<S: ToString>(code: StatusCode, s: S) -> Self {
        Self(code, s.to_string())
    }
}

#[instrument(level = Level::INFO, err(Debug))]
async fn index() -> Result<Html<String>, Response> {
    let file = Assets::get("index.html")
        .ok_or_else(|| Response::new(StatusCode::INTERNAL_SERVER_ERROR, "can't load index.html"))?;
    let data = file.data.into_owned();
    let text = String::from_utf8(data)
        .or_else(|s| Err(Response::new(StatusCode::INTERNAL_SERVER_ERROR, s)))?;
    Ok(Html(text))
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let Response(code, s) = self;
        (code, s).into_response()
    }
}

impl From<OpenAIError> for Response {
    fn from(value: OpenAIError) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, value)
    }
}

#[instrument(level = Level::INFO, skip(state, form), err(Debug))]
async fn summarize(
    State(state): State<AppState>,
    Form(form): Form<SummarizeRequest>,
) -> Result<Html<String>, Response> {
    let config = OpenAIConfig::new().with_api_key(state.openai_api_key);
    let client = Client::with_config(config);
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(format!("Summarize the following text:\n\n{}", form.text))
            .build()?
            .into()])
        .build()?;
    let response = client.chat().create(request).await?;
    let content = response.choices[0].message.content.clone().unwrap();
    Ok(Html(content))
}

pub fn make_router(openai_api_key: String) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/summarize", post(summarize))
        .with_state(AppState { openai_api_key })
}
