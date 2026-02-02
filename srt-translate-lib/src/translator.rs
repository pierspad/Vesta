use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::prompts::{
    build_single_translation_prompt,
    build_batch_translation_prompt,
    build_context_enhanced_translation_prompt,
};

/// Tipo di API da utilizzare
#[derive(Debug, Clone, PartialEq)]
pub enum ApiType {
    Local,
    OpenAI,
    Gemini,
}

/// Configurazione del traduttore
#[derive(Clone)]
pub struct TranslatorConfig {
    /// URL base per l'API (default: http://localhost:1234/v1 per LLM locale)
    pub base_url: String,
    /// Nome del modello (default: local-model)
    pub model: String,
    /// API key opzionale (se presente, usa OpenAI o Gemini, altrimenti LLM locale)
    pub api_key: Option<String>,
    /// Tipo di API da utilizzare
    pub api_type: ApiType,
}

impl Default for TranslatorConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:1234/v1".to_string(),
            model: "local-model".to_string(),
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

    pub async fn translate(&self, text: &str, target_lang: &str, context: Option<&str>) -> Result<String> {
        match self.config.api_type {
            ApiType::Gemini => self.translate_gemini(text, target_lang, context).await,
            _ => self.translate_llm(text, target_lang, context).await,
        }
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
        match self.config.api_type {
            ApiType::Gemini => self.translate_with_context_gemini(text, target_lang, title_context, surrounding_context).await,
            _ => self.translate_with_context_llm(text, target_lang, title_context, surrounding_context).await,
        }
    }

    pub async fn translate_batch(
        &self,
        texts_with_ids: &[(u32, String)],
        target_lang: &str,
        context: Option<&str>,
    ) -> Result<HashMap<u32, String>> {
        match self.config.api_type {
            ApiType::Gemini => self.translate_batch_gemini(texts_with_ids, target_lang, context).await,
            _ => self.translate_batch_llm(texts_with_ids, target_lang, context).await,
        }
    }

    /// Traduzione singola usando API LLM (locale o remota)
    async fn translate_llm(
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
        
        // Aggiungi header Authorization solo se API key è presente (remoto)
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
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

    /// Traduzione batch usando API LLM (locale o remota)
    async fn translate_batch_llm(
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

    /// Traduzione singola usando API Google Gemini
    async fn translate_gemini(
        &self,
        text: &str,
        target_lang: &str,
        context: Option<&str>,
    ) -> Result<String> {
        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct GeminiRequest {
            contents: Vec<Content>,
        }

        #[derive(Deserialize)]
        struct GeminiPart {
            text: String,
        }

        #[derive(Deserialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }

        #[derive(Deserialize)]
        struct GeminiCandidate {
            content: GeminiContent,
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<GeminiCandidate>,
        }

        let prompt = build_single_translation_prompt(text, target_lang, context);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Gemini API key is required"))?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.config.model
        );

        let response = self.client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        Ok(response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.trim().trim_matches('"').to_string())
            .unwrap_or_default())
    }

    /// Traduzione batch usando API Google Gemini
    async fn translate_batch_gemini(
        &self,
        texts_with_ids: &[(u32, String)],
        target_lang: &str,
        context: Option<&str>,
    ) -> Result<HashMap<u32, String>> {
        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct GeminiRequest {
            contents: Vec<Content>,
        }

        #[derive(Deserialize)]
        struct GeminiPart {
            text: String,
        }

        #[derive(Deserialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }

        #[derive(Deserialize)]
        struct GeminiCandidate {
            content: GeminiContent,
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<GeminiCandidate>,
        }

        let prompt = build_batch_translation_prompt(texts_with_ids, target_lang, context);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Gemini API key is required"))?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.config.model
        );

        let response = self.client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        let result_text = response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.trim().to_string())
            .unwrap_or_default();

        // Parse JSON result - molto più robusto del parsing manuale
        let translations = parse_json_translations(&result_text, texts_with_ids.len())?;

        Ok(translations)
    }

    /// Traduzione con contesto migliorato usando API LLM
    async fn translate_with_context_llm(
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

    /// Traduzione con contesto migliorato usando API Gemini
    async fn translate_with_context_gemini(
        &self,
        text: &str,
        target_lang: &str,
        title_context: Option<&str>,
        surrounding_context: Option<&str>,
    ) -> Result<String> {
        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct GeminiRequest {
            contents: Vec<Content>,
        }

        #[derive(Deserialize)]
        struct GeminiPart {
            text: String,
        }

        #[derive(Deserialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }

        #[derive(Deserialize)]
        struct GeminiCandidate {
            content: GeminiContent,
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            #[serde(default)]
            candidates: Vec<GeminiCandidate>,
            #[serde(rename = "promptFeedback")]
            prompt_feedback: Option<PromptFeedback>,
        }

        #[derive(Deserialize)]
        struct PromptFeedback {
            #[serde(rename = "blockReason")]
            block_reason: Option<String>,
        }

        let prompt = build_context_enhanced_translation_prompt(text, target_lang, title_context, surrounding_context);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt,
                }],
            }],
        };

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("API key required for Gemini"))?;

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.base_url.trim_end_matches('/'),
            self.config.model,
            api_key
        );

        // Implementa retry con exponential backoff
        let max_retries = 3;
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            if attempt > 0 {
                // Exponential backoff: 1s, 2s, 4s
                let delay = std::time::Duration::from_secs(2_u64.pow(attempt as u32));
                tokio::time::sleep(delay).await;
            }

            let response_result = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response_result {
                Ok(resp) => {
                    // Prima leggi la risposta come text per debugging
                    let response_text = match resp.text().await {
                        Ok(text) => text,
                        Err(e) => {
                            last_error = Some(anyhow::anyhow!("Failed to read response: {}", e));
                            continue;
                        }
                    };

                    // Prova a parsare come JSON
                    match serde_json::from_str::<GeminiResponse>(&response_text) {
                        Ok(gemini_response) => {
                            // Controlla se la risposta è stata bloccata
                            if gemini_response.candidates.is_empty() {
                                if let Some(feedback) = gemini_response.prompt_feedback {
                                    if let Some(reason) = feedback.block_reason {
                                        // Per contenuti bloccati, usa una traduzione semplice senza contesto
                                        eprintln!("⚠️  Content blocked by safety filters: {}. Using fallback simple translation.", reason);
                                        return self.translate_gemini(text, target_lang, title_context).await;
                                    }
                                }
                                last_error = Some(anyhow::anyhow!("Empty response from API, retrying..."));
                                continue;
                            }

                            // Estrai il testo tradotto
                            return Ok(gemini_response
                                .candidates
                                .first()
                                .and_then(|c| c.content.parts.first())
                                .map(|p| p.text.trim().trim_matches('"').to_string())
                                .unwrap_or_else(|| {
                                    // Fallback: se non c'è testo, ritorna l'originale
                                    text.to_string()
                                }));
                        }
                        Err(e) => {
                            last_error = Some(anyhow::anyhow!(
                                "Failed to parse response (attempt {}/{}): {}. Response: {}",
                                attempt + 1,
                                max_retries,
                                e,
                                &response_text[..response_text.len().min(200)] // primi 200 caratteri
                            ));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Request failed (attempt {}/{}): {}", attempt + 1, max_retries, e));
                    continue;
                }
            }
        }

        // Se tutti i retry falliscono, ritorna l'errore
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Translation failed after {} retries", max_retries)))
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
