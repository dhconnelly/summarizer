use anyhow::anyhow;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use shuttle_secrets::{SecretStore, Secrets};

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

#[derive(Clone)]
struct AppState {
    client: Client<OpenAIConfig>,
}

async fn index() -> Result<Html<String>, StatusCode> {
    let file = Assets::get("index.html").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let data = file.data.into_owned();
    let text = String::from_utf8(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(text))
}

async fn summarize(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("The Los Angeles Dodgers won the World Series in 2020.")
                .build()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("Where was it played?")
                .build()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .into(),
        ])
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = state
        .client
        .chat()
        .create(request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let content = response.choices[0].message.content.clone().unwrap();
    Ok(Html(content))
}

#[shuttle_runtime::main]
async fn main(#[Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let key = secrets
        .get("OPENAI_API_KEY")
        .ok_or(anyhow!("missing OPENAI_API_KEY"))?;
    let config = OpenAIConfig::new().with_api_key(key);
    let client = Client::with_config(config);
    let state = AppState { client };

    let router = Router::new()
        .route("/", get(index))
        .route("/summarize", post(summarize))
        .with_state(state);

    Ok(router.into())
}
