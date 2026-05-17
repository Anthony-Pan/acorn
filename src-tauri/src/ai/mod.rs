pub mod error;
pub mod keychain;
pub mod metadata;
pub mod prompts;
pub mod provider;
pub mod providers;
pub mod transcribe;
pub mod types;

pub use error::ProviderError;
pub use metadata::{provider_catalog, ProviderMetadata};
pub use provider::Provider;
pub use types::{DecomposeEvent, DecomposeRequest, DecomposeResponse};
