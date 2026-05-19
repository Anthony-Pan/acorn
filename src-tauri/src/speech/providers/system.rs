//! Cross-platform router for the "system" speech provider.
//!
//! On macOS this resolves to `SystemSpeechMacosProvider` (real native impl).
//! On every other platform we return `SpeechError::Unavailable` — the
//! catalogue's `supported_platforms` field already prevents users from
//! selecting it, but this is the runtime safety net.

use crate::speech::error::SpeechResult;
use crate::speech::metadata::SpeechProviderMetadata;
use crate::speech::provider::SpeechProvider;

#[cfg(target_os = "macos")]
use super::system_macos::SystemSpeechMacosProvider;

#[cfg(target_os = "macos")]
pub fn build(metadata: SpeechProviderMetadata) -> SpeechResult<Box<dyn SpeechProvider>> {
    Ok(Box::new(SystemSpeechMacosProvider::new(metadata)))
}

#[cfg(not(target_os = "macos"))]
pub fn build(metadata: SpeechProviderMetadata) -> SpeechResult<Box<dyn SpeechProvider>> {
    let _ = metadata;
    Err(crate::speech::error::SpeechError::Unavailable)
}
