use axum::extract::State;
use axum::{
    extract::Query,
    http::header::CONTENT_TYPE,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sbv2_core::tts_util::kata_tone2phone_tone;
use sbv2_core::{
    tts::{SynthesizeOptions, TTSModelHolder},
    tts_util::preprocess_parse_text,
};
use serde::{Deserialize, Serialize};
use tokio::{fs, net::TcpListener, sync::Mutex};

use std::env;
use std::sync::Arc;

use error::AppResult;

mod error;

#[derive(Deserialize)]
struct RequestCreateAudioQuery {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct AudioQuery {
    kana: String,
    tone: i32,
}

#[derive(Serialize)]
struct ResponseCreateAudioQuery {
    audio_query: Vec<AudioQuery>,
    text: String,
}

async fn create_audio_query(
    State(state): State<AppState>,
    Query(request): Query<RequestCreateAudioQuery>,
) -> AppResult<impl IntoResponse> {
    let (text, process) = {
        let tts_model = state.tts_model.lock().await;
        preprocess_parse_text(&request.text, &tts_model.jtalk)?
    };
    let kana_tone_list = process.g2kana_tone()?;
    let audio_query = kana_tone_list
        .iter()
        .map(|(kana, tone)| AudioQuery {
            kana: kana.clone(),
            tone: *tone,
        })
        .collect::<Vec<_>>();
    Ok(Json(ResponseCreateAudioQuery { audio_query, text }))
}

#[derive(Deserialize)]
pub struct RequestSynthesis {
    text: String,
    speaker_id: i64,
    sdp_ratio: f32,
    length_scale: f32,
    style_id: i32,
    audio_query: Vec<AudioQuery>,
    ident: String,
}

async fn synthesis(
    State(state): State<AppState>,
    Json(request): Json<RequestSynthesis>,
) -> AppResult<impl IntoResponse> {
    let phone_tone = request
        .audio_query
        .iter()
        .map(|query| (query.kana.clone(), query.tone))
        .collect::<Vec<_>>();
    let phone_tone = kata_tone2phone_tone(phone_tone);
    let tones = phone_tone.iter().map(|(_, tone)| *tone).collect::<Vec<_>>();
    let buffer = {
        let mut tts_model = state.tts_model.lock().await;
        tts_model.easy_synthesize_neo(
            &request.ident,
            &request.text,
            Some(tones),
            request.style_id,
            request.speaker_id,
            SynthesizeOptions {
                sdp_ratio: request.sdp_ratio,
                length_scale: request.length_scale,
                ..Default::default()
            },
        )?
    };
    Ok(([(CONTENT_TYPE, "audio/wav")], buffer))
}

#[derive(Clone)]
struct AppState {
    tts_model: Arc<Mutex<TTSModelHolder>>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let mut tts_model = TTSModelHolder::new(
            &fs::read(env::var("BERT_MODEL_PATH")?).await?,
            &fs::read(env::var("TOKENIZER_PATH")?).await?,
            env::var("HOLDER_MAX_LOADED_MODElS")
                .ok()
                .and_then(|x| x.parse().ok()),
        )?;
        let models = env::var("MODELS_PATH").unwrap_or("models".to_string());
        let mut f = fs::read_dir(&models).await?;
        let mut entries = vec![];
        while let Ok(Some(e)) = f.next_entry().await {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".onnx") && name.starts_with("model_") {
                let name_len = name.len();
                let name = name.chars();
                entries.push(
                    name.collect::<Vec<_>>()[6..name_len - 5]
                        .iter()
                        .collect::<String>(),
                );
            } else if name.ends_with(".sbv2") {
                let entry = &name[..name.len() - 5];
                log::info!("Try loading: {entry}");
                let sbv2_bytes = match fs::read(format!("{models}/{entry}.sbv2")).await {
                    Ok(b) => b,
                    Err(e) => {
                        log::warn!("Error loading sbv2_bytes from file {entry}: {e}");
                        continue;
                    }
                };
                if let Err(e) = tts_model.load_sbv2file(entry, sbv2_bytes) {
                    log::warn!("Error loading {entry}: {e}");
                };
                log::info!("Loaded: {entry}");
            } else if name.ends_with(".aivmx") {
                let entry = &name[..name.len() - 6];
                log::info!("Try loading: {entry}");
                let aivmx_bytes = match fs::read(format!("{models}/{entry}.aivmx")).await {
                    Ok(b) => b,
                    Err(e) => {
                        log::warn!("Error loading aivmx bytes from file {entry}: {e}");
                        continue;
                    }
                };
                if let Err(e) = tts_model.load_aivmx(entry, aivmx_bytes) {
                    log::error!("Error loading {entry}: {e}");
                }
                log::info!("Loaded: {entry}");
            }
        }
        for entry in entries {
            log::info!("Try loading: {entry}");
            let style_vectors_bytes =
                match fs::read(format!("{models}/style_vectors_{entry}.json")).await {
                    Ok(b) => b,
                    Err(e) => {
                        log::warn!("Error loading style_vectors_bytes from file {entry}: {e}");
                        continue;
                    }
                };
            let vits2_bytes = match fs::read(format!("{models}/model_{entry}.onnx")).await {
                Ok(b) => b,
                Err(e) => {
                    log::warn!("Error loading vits2_bytes from file {entry}: {e}");
                    continue;
                }
            };
            if let Err(e) = tts_model.load(&entry, style_vectors_bytes, vits2_bytes) {
                log::warn!("Error loading {entry}: {e}");
            };
            log::info!("Loaded: {entry}");
        }
        Ok(Self {
            tts_model: Arc::new(Mutex::new(tts_model)),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    env_logger::init();
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/audio_query", get(create_audio_query))
        .route("/synthesis", post(synthesis))
        .with_state(AppState::new().await?);
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
