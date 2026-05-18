// Items become "live" as later commits wire the Tauri commands and frontend.
// Both allows are removed in the command-routing commit.
#![allow(dead_code, unused_imports)]

pub mod error;
pub mod metadata;
pub mod provider;
pub mod providers;
pub mod types;

pub use error::{SpeechError, SpeechResult};
pub use metadata::{
    default_speech_provider_id, find_speech_metadata, speech_provider_catalog, SpeechApiFormat,
    SpeechProviderMetadata, SpeechProviderStatus,
};
pub use provider::{build_speech_provider, SpeechProvider, SpeechProviderInputs};
pub use types::{TranscribeRequest, TranscribeResponse};
