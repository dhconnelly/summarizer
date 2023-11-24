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

async fn index() -> Result<Html<String>, StatusCode> {
    let file = Assets::get("index.html").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let data = file.data.into_owned();
    let text = String::from_utf8(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(text))
}

struct Response(StatusCode, String);

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let Response(code, s) = self;
        (code, s).into_response()
    }
}

impl From<OpenAIError> for Response {
    fn from(value: OpenAIError) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

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
    let state = AppState { openai_api_key };
    Router::new()
        .route("/", get(index))
        .route("/summarize", post(summarize))
        .with_state(state)
}
