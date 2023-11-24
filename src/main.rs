use anyhow::anyhow;
use shuttle_secrets::{SecretStore, Secrets};
use summarizer;

#[shuttle_runtime::main]
async fn main(#[Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let openai_api_key = secrets
        .get("OPENAI_API_KEY")
        .ok_or(anyhow!("missing OPENAI_API_KEY"))?;
    let router = summarizer::make_router(openai_api_key);
    Ok(router.into())
}
