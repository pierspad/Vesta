//! Comandi Tauri per la traduzione di sottotitoli.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;

use srt_parser::SrtParser;
use srt_translate::{
    ApiType, RateLimitConfig, TranslationProgress, Translator, TranslatorConfig,
    translate_subtitles_with_rate_limit_cancellable,
};

use crate::state::AppTranslateState;

/// Configurazione per la traduzione
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateConfig {
    pub input_path: String,
    pub output_path: String,
    pub target_lang: String,
    pub api_keys: Vec<String>,
    pub api_type: String, // "local" or "openrouter"
    pub batch_size: usize,
    pub title_context: Option<String>,
    pub api_url: Option<String>,
    pub model: Option<String>,
}

/// Evento di progresso emesso al frontend
#[derive(Debug, Clone, Serialize)]
pub struct TranslateProgressEvent {
    pub message: String,
    pub current_batch: usize,
    pub total_batches: usize,
    pub percentage: f64,
    pub eta_seconds: Option<f64>,
}

/// Risultato della traduzione
#[derive(Debug, Clone, Serialize)]
pub struct TranslateResult {
    pub success: bool,
    pub message: String,
    pub output_path: Option<String>,
    pub translated_count: usize,
}

/// Imposta la configurazione API
#[tauri::command]
pub async fn set_api_config(
    state: State<'_, AppTranslateState>,
    api_key: String,
    api_type: String,
) -> Result<bool, String> {
    let mut translate_state = state.lock().map_err(|e| e.to_string())?;
    translate_state.api_key = Some(api_key);
    translate_state.api_type = Some(api_type);
    Ok(true)
}

/// Carica un file SRT e ritorna info di base
#[tauri::command]
pub async fn load_srt_for_translate(path: String) -> Result<SrtFileInfo, String> {
    let mut subtitles = SrtParser::parse_file(&path)
        .map_err(|e| format!("Errore nel parsing del file SRT: {}", e))?;

    // Normalizza: riempi buchi nella numerazione con "[...]"
    SrtParser::normalize_subtitles(&mut subtitles);

    let mut sorted: Vec<_> = subtitles.values().collect();
    sorted.sort_by_key(|s| s.id);

    let first_text = sorted.first().map(|s| s.text.clone()).unwrap_or_default();
    let last_text = sorted.last().map(|s| s.text.clone()).unwrap_or_default();

    Ok(SrtFileInfo {
        path,
        subtitle_count: subtitles.len(),
        first_subtitle: first_text,
        last_subtitle: last_text,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SrtFileInfo {
    pub path: String,
    pub subtitle_count: usize,
    pub first_subtitle: String,
    pub last_subtitle: String,
}

/// Avvia la traduzione
#[tauri::command]
pub async fn start_translation(
    app: AppHandle,
    state: State<'_, AppTranslateState>,
    config: TranslateConfig,
) -> Result<TranslateResult, String> {
    // Crea un nuovo cancellation token
    let cancellation_token = CancellationToken::new();
    
    // Controlla se già in traduzione e salva il token
    {
        let mut translate_state = state.lock().map_err(|e| e.to_string())?;
        if translate_state.is_translating {
            return Err("Traduzione già in corso".to_string());
        }
        translate_state.is_translating = true;
        translate_state.cancellation_token = Some(cancellation_token.clone());
    }

    // Esegui la traduzione
    let result = perform_translation(app.clone(), config, cancellation_token.clone()).await;

    // Reset flag traduzione e rimuovi token
    {
        if let Ok(mut translate_state) = state.lock() {
            translate_state.is_translating = false;
            translate_state.cancellation_token = None;
        }
    }

    result
}

async fn perform_translation(
    app: AppHandle,
    config: TranslateConfig,
    cancellation_token: CancellationToken,
) -> Result<TranslateResult, String> {
    // Carica i sottotitoli
    let mut subtitles = SrtParser::parse_file(&config.input_path)
        .map_err(|e| format!("Errore caricamento SRT: {}", e))?;

    // Normalizza: riempi buchi nella numerazione con "[...]"
    SrtParser::normalize_subtitles(&mut subtitles);

    let total_count = subtitles.len();

    // Determina il tipo di API - ora supporta: Local e Google (OpenRouter disabilitato)
    let api_type = match config.api_type.to_lowercase().as_str() {
        "local" => ApiType::Local,
        "google" | "gemini" => ApiType::Google,
        "groq" => ApiType::Groq,
        "custom" => ApiType::Local, // Custom providers use OpenAI-compatible API
        // OpenRouter e altri sono disabilitati per ora
        "openrouter" | "openai" | "anthropic" | "mistral" => {
            return Err("Provider disabilitato. Usa 'google', 'local' o 'groq'.".to_string());
        }
        _ => return Err(format!("Tipo API non supportato: {}. Usa 'google', 'local' o 'groq'.", config.api_type)),
    };

    // Determina URL base sul tipo
    let base_url = config.api_url.unwrap_or_else(|| {
        match api_type {
            ApiType::Local => "http://localhost:11434/v1".to_string(),
            ApiType::Google => "https://generativelanguage.googleapis.com/v1beta".to_string(),
            ApiType::Groq => "https://api.groq.com/openai/v1".to_string(),
            ApiType::OpenRouter => "https://openrouter.ai/api/v1".to_string(), // Non usato
        }
    });

    // Determina il modello - per Google, usa solo il nome del modello senza prefisso
    let model = config.model.unwrap_or_else(|| {
        match api_type {
            ApiType::Local => "llama3.2".to_string(),
            ApiType::Google => "gemini-2.0-flash".to_string(),
            ApiType::Groq => "llama-3.3-70b-versatile".to_string(),
            ApiType::OpenRouter => "google/gemini-2.0-flash-001".to_string(), // Non usato
        }
    });

    // Se la lista chiavi è vuota ma il tipo richiede chiavi (es. Google), errore
    // Per local, possiamo avere lista vuota
    let keys = if config.api_keys.is_empty() {
        if api_type == ApiType::Local {
            vec!["".to_string()] // Empty key for local
        } else {
            return Err("Nessuna chiave API fornita. Aggiungi le tue chiavi API nelle impostazioni.".to_string());
        }
    } else {
        config.api_keys.clone()
    };

    let mut translators = Vec::new();
    let mut rate_limiters = Vec::new();

    // Crea un translator per ogni chiave API (rotazione round-robin)
    for key in keys {
        // Crea il translator con la configurazione corretta
        let translator_config = TranslatorConfig {
            api_type: api_type.clone(),
            api_key: if key.is_empty() { None } else { Some(key) },
            base_url: base_url.clone(),
            model: model.clone(),
        };

        translators.push(Translator::new(translator_config));
        
        // Crea rate limiter per ogni chiave (15 RPM per Gemini free tier, 1000 per pay-as-you-go)
        // Usiamo 15 RPM come default conservativo
        let rate_limit_config = RateLimitConfig::new(15);
        rate_limiters.push(rate_limit_config.create_limiter());
    }


    let output_path = PathBuf::from(&config.output_path);

    // Wrapper per il callback di progresso che emette eventi Tauri
    let app_handle = Arc::new(TokioMutex::new(app.clone()));
    
    let on_progress = {
        let app_handle = app_handle.clone();
        move |progress: TranslationProgress| {
            let percentage = if progress.total_batches > 0 {
                (progress.current_batch as f64 / progress.total_batches as f64) * 100.0
            } else {
                0.0
            };

            let event = TranslateProgressEvent {
                message: progress.message,
                current_batch: progress.current_batch,
                total_batches: progress.total_batches,
                percentage,
                eta_seconds: progress.eta_seconds,
            };

            // Usa tokio spawn per emettere l'evento
            let app_handle = app_handle.clone();
            tokio::spawn(async move {
                if let Ok(app) = app_handle.try_lock() {
                    let _ = app.emit("translate-progress", event);
                }
            });
        }
    };

    // Esegui la traduzione con supporto per cancellazione
    let translated: anyhow::Result<std::collections::HashMap<u32, srt_parser::Subtitle>> = translate_subtitles_with_rate_limit_cancellable(
        translators,
        Some(rate_limiters),
        subtitles,
        &config.target_lang,
        config.batch_size,
        config.title_context.as_deref(),
        &output_path,
        on_progress,
        cancellation_token,
    )
    .await;
    
    // Gestisci la cancellazione
    let translated: std::collections::HashMap<u32, srt_parser::Subtitle> = match translated {
        Ok(t) => t,
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("cancelled") || error_str.contains("annullat") {
                // Emetti evento di cancellazione
                let _ = app.emit("translate-complete", TranslateResult {
                    success: false,
                    message: "Traduzione annullata dall'utente".to_string(),
                    output_path: None,
                    translated_count: 0,
                });
                return Ok(TranslateResult {
                    success: false,
                    message: "Traduzione annullata".to_string(),
                    output_path: None,
                    translated_count: 0,
                });
            }
            return Err(format!("Errore traduzione: {}", e));
        }
    };

    // Emetti evento di completamento
    let _ = app.emit("translate-complete", TranslateResult {
        success: true,
        message: format!("Traduzione completata: {} sottotitoli", translated.len()),
        output_path: Some(config.output_path.clone()),
        translated_count: translated.len(),
    });

    Ok(TranslateResult {
        success: true,
        message: format!("Tradotti {} sottotitoli su {}", translated.len(), total_count),
        output_path: Some(config.output_path),
        translated_count: translated.len(),
    })
}

/// Annulla la traduzione in corso
#[tauri::command]
pub async fn cancel_translation(
    state: State<'_, AppTranslateState>,
) -> Result<bool, String> {
    let mut translate_state = state.lock().map_err(|e| e.to_string())?;
    
    // Cancella il token se presente - questo fermerà tutte le richieste in corso
    if let Some(ref token) = translate_state.cancellation_token {
        token.cancel();
    }
    
    translate_state.is_translating = false;
    translate_state.cancellation_token = None;
    
    Ok(true)
}

/// Rappresenta una coppia di sottotitoli (originale e tradotto)
#[derive(Debug, Clone, Serialize)]
pub struct SubtitlePair {
    pub id: u32,
    pub original: String,
    pub translated: String,
}

/// Legge gli ultimi N sottotitoli dal file di input e output
#[tauri::command]
pub async fn get_latest_translated_subtitles(
    input_path: String,
    output_path: String,
    count: usize,
) -> Result<Vec<SubtitlePair>, String> {
    use std::path::Path;
    
    // Verifica che il file di output esista
    if !Path::new(&output_path).exists() {
        return Ok(vec![]);
    }
    
    // Carica i sottotitoli originali
    let original_subs = SrtParser::parse_file(&input_path)
        .map_err(|e| format!("Errore lettura file originale: {}", e))?;
    
    // Carica i sottotitoli tradotti
    let translated_subs = SrtParser::parse_file(&output_path)
        .map_err(|e| format!("Errore lettura file tradotto: {}", e))?;
    
    // Ottieni gli ID ordinati dei sottotitoli tradotti
    let mut translated_ids: Vec<u32> = translated_subs.keys().cloned().collect();
    translated_ids.sort();
    
    // Prendi gli ultimi N
    let start_idx = if translated_ids.len() > count {
        translated_ids.len() - count
    } else {
        0
    };
    
    let latest_ids = &translated_ids[start_idx..];
    
    // Crea le coppie
    let mut pairs = Vec::new();
    for &id in latest_ids {
        let original_text = original_subs.get(&id)
            .map(|s| s.text.clone())
            .unwrap_or_else(|| "—".to_string());
        let translated_text = translated_subs.get(&id)
            .map(|s| s.text.clone())
            .unwrap_or_else(|| "—".to_string());
        
        pairs.push(SubtitlePair {
            id,
            original: original_text,
            translated: translated_text,
        });
    }
    
    Ok(pairs)
}
