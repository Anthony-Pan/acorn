pub mod error;
pub mod metadata;
pub mod provider;
pub mod providers;
pub mod types;

pub use error::{SpeechError, SpeechResult};
pub use metadata::{
    default_speech_provider_id, find_speech_metadata, speech_provider_catalog,
    SpeechProviderMetadata,
};
pub use provider::{build_speech_provider, SpeechProviderInputs};
pub use types::TranscribeResponse;
