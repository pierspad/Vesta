use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::prompts::{
    build_single_translation_prompt,
    build_batch_translation_prompt,
    build_context_enhanced_translation_prompt,
};

/// Tipo di API da utilizzare - SEMPLIFICATO
/// Tutti usano formato OpenAI-compatible (incluso OpenRouter)
#[derive(Debug, Clone, PartialEq)]
pub enum ApiType {
    /// Server locale (Ollama, LM Studio) - nessuna API key richiesta
    Local,
    /// OpenRouter o qualsiasi API OpenAI-compatible - richiede API key
    OpenRouter,
}

/// Configurazione del traduttore
#[derive(Clone)]
pub struct TranslatorConfig {
    /// URL base per l'API
    /// - Local: http://localhost:11434/v1 (Ollama) o http://localhost:1234/v1 (LM Studio)
    /// - OpenRouter: https://openrouter.ai/api/v1
    pub base_url: String,
    /// Nome del modello (es: llama3.2, google/gemini-2.0-flash-001)
    pub model: String,
    /// API key (richiesta per OpenRouter, opzionale per Local)
    pub api_key: Option<String>,
    /// Tipo di API da utilizzare
    pub api_type: ApiType,
}

impl Default for TranslatorConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            model: "llama3.2".to_string(),
            api_key: None,
            api_type: ApiType::Local,
        }
    }
}

#[derive(Clone)]
pub struct Translator {
    config: TranslatorConfig,
    client: reqwest::Client,
}

impl Translator {
    pub fn new(config: TranslatorConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Traduce un singolo testo - usa sempre formato OpenAI-compatible
    pub async fn translate(&self, text: &str, target_lang: &str, context: Option<&str>) -> Result<String> {
        self.translate_openai(text, target_lang, context).await
    }

    /// Traduce un singolo sottotitolo con contesto aggiuntivo dai sottotitoli circostanti
    /// Usato principalmente per il repair di sottotitoli mancanti
    pub async fn translate_with_context(
        &self,
        text: &str,
        target_lang: &str,
        title_context: Option<&str>,
        surrounding_context: Option<&str>,
    ) -> Result<String> {
        self.translate_with_context_openai(text, target_lang, title_context, surrounding_context).await
    }

    /// Traduce un batch di testi - usa sempre formato OpenAI-compatible
    pub async fn translate_batch(
        &self,
        texts_with_ids: &[(u32, String)],
        target_lang: &str,
        context: Option<&str>,
    ) -> Result<HashMap<u32, String>> {
        self.translate_batch_openai(texts_with_ids, target_lang, context).await
    }

    /// Traduzione singola usando API OpenAI-compatible (funziona con Local, OpenRouter, etc.)
    async fn translate_openai(
        &self,
        text: &str,
        target_lang: &str,
        context: Option<&str>,
    ) -> Result<String> {
        #[derive(Serialize, Deserialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct Request {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }

        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }

        let prompt = build_single_translation_prompt(text, target_lang, context);

        let request = Request {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.3,
        };

        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        let mut req_builder = self.client.post(&url).json(&request);
        
        // Aggiungi header Authorization solo se API key è presente
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Aggiungi header specifici per OpenRouter
        if self.config.api_type == ApiType::OpenRouter {
            req_builder = req_builder
                .header("HTTP-Referer", "https://srt-tools.app")
                .header("X-Title", "SRT Tools");
        }

        let response = req_builder
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(response
            .choices
            .first()
            .map(|c| c.message.content.trim().trim_matches('"').to_string())
            .unwrap_or_default())
    }

    /// Traduzione batch usando API OpenAI-compatible
    async fn translate_batch_openai(
        &self,
        texts_with_ids: &[(u32, String)],
        target_lang: &str,
        context: Option<&str>,
    ) -> Result<HashMap<u32, String>> {
        #[derive(Serialize, Deserialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct Request {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }

        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }

        let prompt = build_batch_translation_prompt(texts_with_ids, target_lang, context);

        let request = Request {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.3,
        };

        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Aggiungi header specifici per OpenRouter
        if self.config.api_type == ApiType::OpenRouter {
            req_builder = req_builder
                .header("HTTP-Referer", "https://srt-tools.app")
                .header("X-Title", "SRT Tools");
        }

        let response = req_builder
            .send()
            .await?
            .json::<Response>()
            .await?;

        let result_text = response
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default();

        // Parse JSON result - molto più robusto del parsing manuale
        let translations = parse_json_translations(&result_text, texts_with_ids.len())?;

        Ok(translations)
    }

    /// Traduzione con contesto migliorato usando API OpenAI-compatible
    async fn translate_with_context_openai(
        &self,
        text: &str,
        target_lang: &str,
        title_context: Option<&str>,
        surrounding_context: Option<&str>,
    ) -> Result<String> {
        #[derive(Serialize, Deserialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct Request {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }

        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }

        let prompt = build_context_enhanced_translation_prompt(text, target_lang, title_context, surrounding_context);

        let request = Request {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.3,
        };

        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Aggiungi header specifici per OpenRouter
        if self.config.api_type == ApiType::OpenRouter {
            req_builder = req_builder
                .header("HTTP-Referer", "https://srt-tools.app")
                .header("X-Title", "SRT Tools");
        }

        let response = req_builder
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(response
            .choices
            .first()
            .map(|c| c.message.content.trim().trim_matches('"').to_string())
            .unwrap_or_default())
    }
}

/// Struttura per deserializzare le traduzioni JSON dall'LLM
#[derive(Deserialize, Debug)]
struct TranslationItem {
    id: u32,
    text: String,
}

/// Parsa la risposta JSON dell'LLM in una HashMap di traduzioni
/// 
/// Questa funzione è robusta e gestisce:
/// - JSON puro
/// - JSON racchiuso in code blocks markdown (```json ... ```)
/// - Variazioni minori nel formato
fn parse_json_translations(response: &str, expected_count: usize) -> Result<HashMap<u32, String>> {
    // Rimuovi eventuali code blocks markdown
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    
    // Trova l'array JSON (cerca '[' e ']')
    let json_start = cleaned.find('[');
    let json_end = cleaned.rfind(']');
    
    let json_str = match (json_start, json_end) {
        (Some(start), Some(end)) if end > start => &cleaned[start..=end],
        _ => cleaned, // Prova comunque con l'intero contenuto
    };
    
    // Prova a parsare come array di TranslationItem
    match serde_json::from_str::<Vec<TranslationItem>>(json_str) {
        Ok(items) => {
            let translations: HashMap<u32, String> = items
                .into_iter()
                .map(|item| (item.id, item.text))
                .collect();
            
            // Verifica che abbiamo tutte le traduzioni
            if translations.len() != expected_count {
                anyhow::bail!(
                    "Batch translation incomplete: expected {} translations, got {}",
                    expected_count,
                    translations.len()
                );
            }
            
            Ok(translations)
        }
        Err(e) => {
            // Fallback: prova parsing legacy per retrocompatibilità
            // (nel caso l'LLM risponda con il vecchio formato)
            if let Some(translations) = try_legacy_parsing(cleaned, expected_count) {
                return Ok(translations);
            }
            
            anyhow::bail!(
                "Failed to parse JSON response: {}. Response was: {}",
                e,
                &response[..response.len().min(500)]
            )
        }
    }
}

/// Fallback al parsing legacy (ID: X | TRANSLATION: Y) per retrocompatibilità
fn try_legacy_parsing(text: &str, expected_count: usize) -> Option<HashMap<u32, String>> {
    let mut translations = HashMap::new();
    let mut current_id: Option<u32> = None;
    let mut current_translation = String::new();
    
    for line in text.lines() {
        let line_lower = line.to_lowercase();
        // Supporta varianti: "ID:", "id:", "Subtitle ID:", etc.
        if line_lower.starts_with("id:") || line_lower.contains("id:") {
            // Salva la traduzione precedente se esiste
            if let Some(id) = current_id {
                translations.insert(id, current_translation.trim().to_string());
            }
            
            // Cerca il pattern ID e TRANSLATION
            if let Some((id_part, trans_part)) = line.split_once('|') {
                // Estrai l'ID numerico
                let id_str: String = id_part.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(id) = id_str.parse::<u32>() {
                    current_id = Some(id);
                    // Rimuovi eventuali prefissi come "TRANSLATION:" (case-insensitive)
                    let trans = trans_part
                        .trim()
                        .trim_start_matches(|c: char| !c.is_alphabetic() || c.is_ascii_uppercase())
                        .trim_start_matches("TRANSLATION:")
                        .trim_start_matches("translation:")
                        .trim_start_matches("Translation:")
                        .trim();
                    current_translation = trans.to_string();
                }
            }
        } else if current_id.is_some() && !line.trim().is_empty() {
            // Aggiungi riga alla traduzione corrente
            if !current_translation.is_empty() {
                current_translation.push('\n');
            }
            current_translation.push_str(line);
        }
    }
    
    // Salva l'ultima traduzione
    if let Some(id) = current_id {
        translations.insert(id, current_translation.trim().to_string());
    }
    
    if translations.len() >= expected_count {
        Some(translations)
    } else {
        None
    }
}
