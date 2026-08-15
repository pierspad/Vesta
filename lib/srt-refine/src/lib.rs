use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use srt_translate::{ApiType, Translator, TranslatorConfig};

pub mod engine;
pub use engine::{RefineEvent, RefineRunConfig, RefineRunSummary, refine_cards_tiered};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineCard {
    pub id: String,

    pub expression: String,

    pub meaning: String,

    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineUpdate {
    pub id: String,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefineLlmConfig {
    pub api_type: String,
    pub api_key: Option<String>,
    pub api_url: Option<String>,
    pub model: Option<String>,
}

fn backup_path() -> PathBuf {
    std::env::temp_dir().join("vesta_refine_backup.tmp")
}

pub fn analyze_tsv_columns(rows: &[Vec<String>]) -> Vec<usize> {
    if rows.is_empty() {
        return Vec::new();
    }
    let col_count = rows[0].len();
    let mut text_cols = Vec::new();

    for col_idx in 0..col_count {
        let mut is_media = false;
        let mut is_sequence = false;

        for row in rows.iter().take(10) {
            if col_idx >= row.len() {
                continue;
            }
            let cell_trimmed = row[col_idx].trim();

            if cell_trimmed.starts_with("[sound:") && cell_trimmed.ends_with(']') {
                is_media = true;
                break;
            }
            if cell_trimmed.starts_with("<img") && cell_trimmed.ends_with('>') {
                is_media = true;
                break;
            }

            if cell_trimmed.contains('_')
                && (cell_trimmed.contains(':') || cell_trimmed.len() == 16)
                && cell_trimmed.chars().any(|c| c.is_numeric())
            {
                is_sequence = true;
            }
        }

        if !is_media && !is_sequence {
            text_cols.push(col_idx);
        }
    }

    text_cols
}

#[derive(Deserialize)]
struct AnkiField {
    name: String,
    ord: usize,
}

#[derive(Deserialize)]
struct AnkiModel {
    #[allow(dead_code)]
    id: i64,
    #[allow(dead_code)]
    name: String,
    flds: Vec<AnkiField>,
}

fn field_indices(model: Option<&AnkiModel>, field_count: usize) -> (usize, usize, usize) {
    let mut expr_idx = 0;
    let mut mean_idx = 1;
    let mut notes_idx = field_count.saturating_sub(1);

    if let Some(model) = model {
        for field in &model.flds {
            match field.name.to_lowercase().as_str() {
                "expression" | "front" | "target" | "question" => expr_idx = field.ord,
                "meaning" | "back" | "native" | "translation" | "answer" => mean_idx = field.ord,
                "notes" | "note" | "comment" | "spiegazione" => notes_idx = field.ord,
                _ => {}
            }
        }
    }

    (expr_idx, mean_idx, notes_idx)
}

fn read_anki_models(conn: &rusqlite::Connection) -> Result<HashMap<String, AnkiModel>, String> {
    let models_json: String = conn
        .query_row("SELECT models FROM col LIMIT 1", [], |row| row.get(0))
        .map_err(|e| format!("Errore lettura metadati modelli Anki: {e}"))?;

    serde_json::from_str(&models_json)
        .map_err(|e| format!("Errore nel parsing del modello Anki: {e}"))
}

pub fn load_cards(path: &str) -> Result<Vec<RefineCard>, String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err("Il file specificato non esiste".to_string());
    }

    if let Err(e) = fs::copy(&path_buf, backup_path()) {
        eprintln!("Failed to create backup copy: {e}");
    }

    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "tsv" => load_cards_tsv(&path_buf),
        "apkg" => load_cards_apkg(path),
        _ => Err("Formato file non supportato. Usa .tsv o .apkg".to_string()),
    }
}

fn load_cards_tsv(path: &Path) -> Result<Vec<RefineCard>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Impossibile leggere il file TSV: {e}"))?;

    let mut rows = Vec::new();
    for line in content.lines() {
        let cells: Vec<String> = line.split('\t').map(str::to_string).collect();
        if !cells.is_empty() && !cells[0].trim().is_empty() {
            rows.push(cells);
        }
    }

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let text_cols = analyze_tsv_columns(&rows);
    let expr_idx = text_cols.first().copied().unwrap_or(0);
    let mean_idx = text_cols.get(1).copied().unwrap_or(1);

    let notes_idx = if text_cols.len() >= 3 {
        *text_cols.last().unwrap()
    } else {
        999
    };

    let cards = rows
        .iter()
        .enumerate()
        .map(|(idx, row)| RefineCard {
            id: idx.to_string(),
            expression: row.get(expr_idx).cloned().unwrap_or_default(),
            meaning: row.get(mean_idx).cloned().unwrap_or_default(),
            notes: row.get(notes_idx).cloned().unwrap_or_default(),
        })
        .collect();

    Ok(cards)
}

fn load_cards_apkg(path: &str) -> Result<Vec<RefineCard>, String> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Impossibile creare la directory temporanea: {e}"))?;

    srt_apkg::unzip_to(Path::new(path), temp_dir.path())?;

    let db_path = temp_dir.path().join("collection.anki2");
    if !db_path.exists() {
        return Err("Archivio APKG non valido: collection.anki2 mancante".to_string());
    }

    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("Impossibile connettersi al database Anki: {e}"))?;

    let models = read_anki_models(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, mid, flds FROM notes")
        .map_err(|e| format!("Errore nella preparazione query SQLite: {e}"))?;

    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Errore nell'esecuzione query SQLite: {e}"))?;

    let mut cards = Vec::new();

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let mid: i64 = row.get(1).map_err(|e| e.to_string())?;
        let flds: String = row.get(2).map_err(|e| e.to_string())?;

        let fields: Vec<String> = flds.split('\x1f').map(str::to_string).collect();
        let (expr_idx, mean_idx, notes_idx) =
            field_indices(models.get(&mid.to_string()), fields.len());

        cards.push(RefineCard {
            id: id.to_string(),
            expression: fields.get(expr_idx).cloned().unwrap_or_default(),
            meaning: fields.get(mean_idx).cloned().unwrap_or_default(),
            notes: fields.get(notes_idx).cloned().unwrap_or_default(),
        });
    }

    Ok(cards)
}

pub fn save_cards(
    input_path: &str,
    output_path: &str,
    updates: Vec<RefineUpdate>,
) -> Result<(), String> {
    let input_path_buf = PathBuf::from(input_path);

    let resolved_input_path = if input_path_buf.exists() {
        input_path_buf
    } else {
        let backup = backup_path();
        if backup.exists() {
            backup
        } else {
            return Err(
                "Il file di input originale non esiste e non è stata trovata alcuna copia cache di backup."
                    .to_string(),
            );
        }
    };

    let output_path_buf = PathBuf::from(output_path);
    if let Some(parent) = output_path_buf.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(format!(
            "La cartella di destinazione '{}' non esiste.",
            parent.display()
        ));
    }

    let ext_of = |p: &str| {
        PathBuf::from(p)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
    };
    let input_ext = ext_of(input_path);
    let output_ext = ext_of(output_path);

    match (input_ext.as_str(), output_ext.as_str()) {
        ("tsv", "tsv") => save_tsv_to_tsv(&resolved_input_path, output_path, updates),
        ("apkg", "tsv") => save_apkg_to_tsv(&resolved_input_path, input_path, output_path, updates),
        ("apkg", "apkg") => {
            save_apkg_to_apkg(&resolved_input_path, input_path, output_path, updates)
        }
        (_, "tsv") => Err("Formato file di input non supportato per esportazione TSV".to_string()),
        (_, "apkg") => {
            Err("Salvare un file TSV come APKG non è supportato in questa scheda.".to_string())
        }
        _ => Err("Formato file non supportato. Usa .tsv o .apkg".to_string()),
    }
}

fn save_tsv_to_tsv(
    input: &Path,
    output_path: &str,
    updates: Vec<RefineUpdate>,
) -> Result<(), String> {
    let content = fs::read_to_string(input)
        .map_err(|e| format!("Impossibile leggere il file TSV di input: {e}"))?;

    let mut rows: Vec<Vec<String>> = content
        .lines()
        .map(|line| line.split('\t').map(str::to_string).collect())
        .collect();

    if rows.is_empty() {
        return Err("Il file TSV è vuoto".to_string());
    }

    let text_cols = analyze_tsv_columns(&rows);

    let notes_idx = if text_cols.len() >= 3 {
        *text_cols.last().unwrap()
    } else {
        return Err("Impossibile identificare la colonna Notes nel TSV".to_string());
    };

    let updates_map: HashMap<usize, String> = updates
        .into_iter()
        .filter_map(|u| u.id.parse::<usize>().ok().map(|idx| (idx, u.notes)))
        .collect();

    for (idx, row) in rows.iter_mut().enumerate() {
        if let Some(new_notes) = updates_map.get(&idx) {
            while row.len() <= notes_idx {
                row.push(String::new());
            }
            row[notes_idx] = new_notes.clone();
        }
    }

    let mut output_content = String::new();
    for row in rows {
        output_content.push_str(&row.join("\t"));
        output_content.push('\n');
    }

    fs::write(output_path, output_content)
        .map_err(|e| format!("Impossibile scrivere il file TSV di output: {e}"))
}

fn save_apkg_to_tsv(
    resolved_input: &Path,
    original_input: &str,
    output_path: &str,
    updates: Vec<RefineUpdate>,
) -> Result<(), String> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Impossibile creare la directory temporanea: {e}"))?;

    let input_path_str = resolved_input.to_str().unwrap_or(original_input);
    srt_apkg::unzip_to(Path::new(input_path_str), temp_dir.path())?;

    let db_path = temp_dir.path().join("collection.anki2");
    if !db_path.exists() {
        return Err("File di input APKG non valido".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Impossibile connettersi al database Anki: {e}"))?;

    let models = read_anki_models(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, mid, flds FROM notes")
        .map_err(|e| format!("Errore preparazione query note Anki: {e}"))?;

    let note_rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("Errore esecuzione query note Anki: {e}"))?;

    let mut cards = Vec::new();
    for note in note_rows.flatten() {
        let (id, mid, flds) = note;
        let fields: Vec<String> = flds.split('\x1f').map(str::to_string).collect();
        let (expr_idx, mean_idx, notes_idx) =
            field_indices(models.get(&mid.to_string()), fields.len());

        cards.push(RefineCard {
            id: id.to_string(),
            expression: fields.get(expr_idx).cloned().unwrap_or_default(),
            meaning: fields.get(mean_idx).cloned().unwrap_or_default(),
            notes: fields.get(notes_idx).cloned().unwrap_or_default(),
        });
    }

    let updates_map: HashMap<String, String> =
        updates.into_iter().map(|u| (u.id, u.notes)).collect();

    let mut output_content = String::new();
    for card in cards {
        let updated_notes = updates_map.get(&card.id).cloned().unwrap_or(card.notes);
        output_content.push_str(&format!(
            "{}\t{}\t{}\n",
            card.expression.replace('\n', "<br>").replace('\t', " "),
            card.meaning.replace('\n', "<br>").replace('\t', " "),
            updated_notes.replace('\n', "<br>").replace('\t', " ")
        ));
    }

    fs::write(output_path, output_content)
        .map_err(|e| format!("Impossibile scrivere il file TSV di output: {e}"))
}

fn save_apkg_to_apkg(
    resolved_input: &Path,
    original_input: &str,
    output_path: &str,
    updates: Vec<RefineUpdate>,
) -> Result<(), String> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Impossibile creare la directory temporanea: {e}"))?;

    let input_path_str = resolved_input.to_str().unwrap_or(original_input);
    srt_apkg::unzip_to(Path::new(input_path_str), temp_dir.path())?;

    let db_path = temp_dir.path().join("collection.anki2");
    if !db_path.exists() {
        return Err("File di input APKG non valido".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Impossibile connettersi al database Anki: {e}"))?;

    let models = read_anki_models(&conn)?;

    let updates_map: HashMap<i64, String> = updates
        .into_iter()
        .filter_map(|u| u.id.parse::<i64>().ok().map(|nid| (nid, u.notes)))
        .collect();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut select_stmt = conn
        .prepare("SELECT mid, flds FROM notes WHERE id = ?")
        .map_err(|e| format!("Errore preparazione query SELECT: {e}"))?;
    let mut update_stmt = conn
        .prepare("UPDATE notes SET flds = ?, sfld = ?, csum = ?, mod = ? WHERE id = ?")
        .map_err(|e| format!("Errore preparazione query UPDATE: {e}"))?;

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    for (&nid, new_notes) in &updates_map {
        let (mid, flds): (i64, String) =
            match select_stmt.query_row([nid], |row| Ok((row.get(0)?, row.get(1)?))) {
                Ok(res) => res,
                Err(_) => continue,
            };

        let mut fields: Vec<String> = flds.split('\x1f').map(str::to_string).collect();
        let (expr_idx, _, notes_idx) = field_indices(models.get(&mid.to_string()), fields.len());

        while fields.len() <= notes_idx {
            fields.push(String::new());
        }
        fields[notes_idx] = new_notes.clone();

        let joined_flds = fields.join("\x1f");
        let sfld = fields.get(expr_idx).map(String::as_str).unwrap_or("");

        let csum = {
            let bytes = sha1_smol::Sha1::from(sfld).digest().bytes();
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64
        };

        update_stmt
            .execute(rusqlite::params![joined_flds, sfld, csum, timestamp, nid])
            .map_err(|e| format!("Errore durante l'aggiornamento SQLite: {e}"))?;
    }

    drop(select_stmt);
    drop(update_stmt);
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    drop(conn);

    srt_apkg::zip_from_dir(temp_dir.path(), Path::new(output_path))
}

pub async fn refine_card_llm(
    card: &RefineCard,
    prompt: &str,
    config: RefineLlmConfig,
) -> Result<String, String> {
    let api_type = match config.api_type.to_lowercase().as_str() {
        "local" => ApiType::Local,
        "google" | "gemini" => ApiType::Google,
        "groq" => ApiType::Groq,
        "custom" => ApiType::Local,
        _ => return Err(format!("Tipo API non supportato: {}", config.api_type)),
    };

    let base_url = config.api_url.unwrap_or_else(|| {
        match api_type {
            ApiType::Local => "http://localhost:11434/v1",
            ApiType::Google => "https://generativelanguage.googleapis.com/v1beta",
            ApiType::Groq => "https://api.groq.com/openai/v1",
            ApiType::OpenRouter => "https://openrouter.ai/api/v1",
        }
        .to_string()
    });

    let model = config.model.unwrap_or_else(|| {
        match api_type {
            ApiType::Local => "llama3.2",
            ApiType::Google => "gemini-2.0-flash",
            ApiType::Groq => "llama-3.3-70b-versatile",
            ApiType::OpenRouter => "google/gemini-2.0-flash-001",
        }
        .to_string()
    });

    let api_key = match &config.api_key {
        None => {
            if api_type == ApiType::Local {
                None
            } else {
                return Err("Chiave API mancante".to_string());
            }
        }
        Some(k) if k.is_empty() => {
            if api_type == ApiType::Local {
                None
            } else {
                return Err("Chiave API mancante".to_string());
            }
        }
        Some(_) => config.api_key.clone(),
    };

    let translator = Translator::new(TranslatorConfig {
        api_type,
        api_key,
        base_url,
        model,
    });

    translator
        .generate_response(&interpolate_prompt(prompt, card))
        .await
        .map_err(|e| format!("Errore chiamata LLM: {e}"))
}

pub fn interpolate_prompt(template: &str, card: &RefineCard) -> String {
    let expression = strip_html(&card.expression);
    let meaning = strip_html(&card.meaning);
    template
        .replace("{{expression}}", &expression)
        .replace("{{front}}", &expression)
        .replace("{{meaning}}", &meaning)
        .replace("{{back}}", &meaning)
        .replace("{{notes}}", &card.notes)
}

pub fn strip_html(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains('<') {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    std::borrow::Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_tsv_columns() {
        let rows = vec![
            vec![
                "Hello world".to_string(),                  // Text (col 0)
                "Ciao mondo".to_string(),                   // Text (col 1)
                "[sound:ep01_0001.mp3]".to_string(),        // Sound media (col 2)
                "<img src=\"ep01_0001.webp\">".to_string(), // Image media (col 3)
                "001_0001_00:00:01".to_string(),            // Sequence (col 4)
                "Some extra notes".to_string(),             // Text (col 5)
            ],
            vec![
                "Second line".to_string(),
                "Seconda riga".to_string(),
                "[sound:ep01_0002.mp3]".to_string(),
                "<img src=\"ep01_0002.webp\">".to_string(),
                "001_0002_00:00:05".to_string(),
                "".to_string(),
            ],
        ];

        let text_cols = analyze_tsv_columns(&rows);
        assert_eq!(text_cols, vec![0, 1, 5]);
    }

    #[test]
    fn test_interpolate_prompt() {
        let card = RefineCard {
            id: "1".to_string(),
            expression: "<b>Bonjour</b>".to_string(),
            meaning: "<i>Hello</i>".to_string(),
            notes: "existing note".to_string(),
        };

        let template =
            "Explain '{{expression}}' (meaning: '{{meaning}}'). Existing notes: {{notes}}";
        let result = interpolate_prompt(template, &card);
        assert_eq!(
            result,
            "Explain 'Bonjour' (meaning: 'Hello'). Existing notes: existing note"
        );
    }

    #[test]
    fn test_strip_html_cow() {
        let plain = "Hello world";
        let res = strip_html(plain);
        assert!(matches!(res, std::borrow::Cow::Borrowed(_)));
        assert_eq!(res, "Hello world");

        let tagged = "Hello <b>world</b>!";
        let res_tagged = strip_html(tagged);
        assert!(matches!(res_tagged, std::borrow::Cow::Owned(_)));
        assert_eq!(res_tagged, "Hello world!");
    }
}
