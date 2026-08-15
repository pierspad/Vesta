use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::LazyLock;

pub const DEFAULT_URL: &str = "http://127.0.0.1:8765";

pub const API_VERSION: u32 = 6;

/// Client HTTP condiviso: riusa il connection pool interno di reqwest invece
/// di aprire un nuovo socket per ogni chiamata ad AnkiConnect.
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkiNote {
    pub deck_name: String,

    pub model_name: String,

    pub fields: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

async fn invoke(url: &str, action: &str, params: Value) -> Result<Value, String> {
    let body = json!({ "action": action, "version": API_VERSION, "params": params });

    let response = HTTP_CLIENT
        .post(url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            format!("AnkiConnect non raggiungibile su {url}: {e}. Anki è aperto e il plugin AnkiConnect è installato?")
        })?;

    let payload: Value = response
        .json()
        .await
        .map_err(|e| format!("Risposta AnkiConnect non valida: {e}"))?;

    if let Some(err) = payload.get("error").filter(|e| !e.is_null()) {
        return Err(format!("AnkiConnect: {err}"));
    }
    Ok(payload.get("result").cloned().unwrap_or(Value::Null))
}

pub async fn ping(url: &str) -> Result<u32, String> {
    let result = invoke(url, "version", json!({})).await?;
    result
        .as_u64()
        .map(|v| v as u32)
        .ok_or_else(|| "Versione AnkiConnect non valida".to_string())
}

pub async fn deck_names(url: &str) -> Result<Vec<String>, String> {
    let result = invoke(url, "deckNames", json!({})).await?;
    serde_json::from_value(result).map_err(|e| format!("deckNames: {e}"))
}

pub async fn import_package(url: &str, apkg_path: &str) -> Result<(), String> {
    let result = invoke(url, "importPackage", json!({ "path": apkg_path })).await?;
    match result.as_bool() {
        Some(true) => Ok(()),
        _ => Err("Import fallito: Anki ha rifiutato il pacchetto".to_string()),
    }
}

pub async fn create_deck(url: &str, name: &str) -> Result<(), String> {
    invoke(url, "createDeck", json!({ "deck": name })).await?;
    Ok(())
}

pub async fn add_notes(url: &str, notes: &[AnkiNote]) -> Result<Vec<Option<i64>>, String> {
    let payload: Vec<Value> = notes
        .iter()
        .map(|n| {
            json!({
                "deckName": n.deck_name,
                "modelName": n.model_name,
                "fields": n.fields,
                "tags": n.tags,
                "options": { "allowDuplicate": false },
            })
        })
        .collect();

    let result = invoke(url, "addNotes", json!({ "notes": payload })).await?;
    serde_json::from_value(result).map_err(|e| format!("addNotes: {e}"))
}

pub async fn store_media_file(url: &str, filename: &str, data_base64: &str) -> Result<(), String> {
    invoke(
        url,
        "storeMediaFile",
        json!({ "filename": filename, "data": data_base64 }),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anki_note_serialization() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("Expression".to_string(), "Hello".to_string());
        fields.insert("Meaning".to_string(), "Ciao".to_string());

        let note = AnkiNote {
            deck_name: "Test Deck".to_string(),
            model_name: "Basic".to_string(),
            fields,
            tags: vec!["vesta".to_string(), "chapter1".to_string()],
        };

        let json = serde_json::to_string(&note).unwrap();
        assert!(json.contains("Test Deck"));
        assert!(json.contains("Hello"));
        assert!(json.contains("vesta"));

        let deserialized: AnkiNote = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.deck_name, "Test Deck");
        assert_eq!(deserialized.tags.len(), 2);
    }

    #[test]
    fn test_add_notes_result_deserialization_with_null_duplicates() {
        // AnkiConnect returns null for duplicates when allowDuplicate is false
        let mock_result = json!([17000000001i64, null, 17000000002i64]);
        let parsed: Vec<Option<i64>> = serde_json::from_value(mock_result).unwrap();

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], Some(17000000001i64));
        assert_eq!(parsed[1], None);
        assert_eq!(parsed[2], Some(17000000002i64));
    }
}
