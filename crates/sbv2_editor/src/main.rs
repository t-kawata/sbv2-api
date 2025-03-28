use axum::{extract::Query, response::IntoResponse, routing::get, Json, Router};
use sbv2_core::{jtalk::JTalk, tts_util::preprocess_parse_text};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use error::AppResult;

mod error;

#[derive(Deserialize)]
struct RequestCreateAudioQuery {
    text: String,
}

#[derive(Serialize)]
struct ResponseCreateAudioQuery {
    kana: String,
    tone: i32,
}

async fn create_audio_query(
    Query(request): Query<RequestCreateAudioQuery>,
) -> AppResult<impl IntoResponse> {
    let (_, process) = preprocess_parse_text(&request.text, &JTalk::new()?)?;
    let kana_tone_list = process.g2kana_tone()?;
    let response = kana_tone_list
        .iter()
        .map(|(kana, tone)| ResponseCreateAudioQuery {
            kana: kana.clone(),
            tone: *tone,
        })
        .collect::<Vec<_>>();
    Ok(Json(response))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/audio_query", get(create_audio_query));
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
