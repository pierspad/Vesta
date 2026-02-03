//! Gestione dello stato globale dell'applicazione.

use std::sync::Mutex;
use std::sync::Arc;
use srt_sync_lib::SyncEngine;
use tokio_util::sync::CancellationToken;

/// Stato per la sincronizzazione sottotitoli
pub struct SyncState {
    pub engine: Option<SyncEngine>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self { engine: None }
    }
}

/// Wrapper thread-safe per lo stato di sincronizzazione
pub type AppSyncState = Mutex<SyncState>;

/// Stato per la traduzione (configurazione)
pub struct TranslateState {
    pub api_key: Option<String>,
    pub api_type: Option<String>,
    pub is_translating: bool,
    pub cancellation_token: Option<CancellationToken>,
}

impl Default for TranslateState {
    fn default() -> Self {
        Self {
            api_key: None,
            api_type: None,
            is_translating: false,
            cancellation_token: None,
        }
    }
}

/// Wrapper thread-safe per lo stato di traduzione
pub type AppTranslateState = Mutex<TranslateState>;
