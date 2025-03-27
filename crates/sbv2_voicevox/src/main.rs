use axum::{routing::get, Router, extract::Query};
use sbv2_core::{jtalk::JTalk, tts_util::preprocess_parse_text};
use tokio::net::TcpListener;
use serde::Deserialize;

use error::AppResult;

mod error;

#[derive(Deserialize)]
struct RequestCreateAudioQuery {
    text: String,
}

async fn create_audio_query(
    Query(request): Query<RequestCreateAudioQuery>,
) -> AppResult<()> {
    let (phones, tones, mut word2ph, normalized_text, process) = preprocess_parse_text(&request.text, &JTalk::new()?)?;
    println!("{:?}", phones);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
