use serde::Serialize;

/// Platform identifiers used in `SpeechProviderMetadata::supported_platforms`.
/// Kept as `&str` so consumers can match `cfg!(target_os = ...)` values directly.
pub const PLATFORM_MACOS: &str = "macos";
pub const PLATFORM_WINDOWS: &str = "windows";
pub const PLATFORM_LINUX: &str = "linux";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeechApiFormat {
    /// Operating-system-native recognition (SFSpeechRecognizer on macOS,
    /// Windows.Media.SpeechRecognition on Windows, unavailable on Linux).
    System,
    /// OpenAI Whisper API (cloud).
    WhisperOpenAi,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeechProviderStatus {
    Available,
    PlatformUnsupported,
    ComingSoon,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechProviderMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub api_format: SpeechApiFormat,
    /// Which OS platforms this provider runs on.
    pub supported_platforms: &'static [&'static str],
    /// True when transcription happens fully on the user's machine.
    pub on_device: bool,
    /// True if the provider requires an API key.
    pub requires_api_key: bool,
    /// When `requires_api_key`, the keychain entry id (typically an AI provider id
    /// such as "openai") that stores the key. Reusing AI provider keys avoids asking
    /// the user to enter the same OpenAI key twice.
    pub api_key_provider_id: Option<&'static str>,
    pub featured: bool,
}

impl SpeechProviderMetadata {
    /// True if this provider is available on the current build target.
    pub fn available_on_current_platform(&self) -> bool {
        self.supported_platforms.contains(&current_platform())
    }
}

/// Returns the OS identifier for the current build target.
pub fn current_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        PLATFORM_MACOS
    }
    #[cfg(target_os = "windows")]
    {
        PLATFORM_WINDOWS
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        PLATFORM_LINUX
    }
}

/// Catalogue of every speech provider Acorn knows about.
///
/// Mirrors `ai::metadata::provider_catalog` in spirit: the catalogue is the
/// single source of truth for what the user can pick in Settings; concrete
/// implementations live in `speech::providers::*` and are routed by
/// `SpeechApiFormat`.
pub fn speech_provider_catalog() -> Vec<SpeechProviderMetadata> {
    vec![
        SpeechProviderMetadata {
            id: "system",
            display_name: "System (Built-in)",
            description: "Uses your operating system's on-device speech recognition. \
                          No API key required, audio never leaves your machine.",
            api_format: SpeechApiFormat::System,
            supported_platforms: &[PLATFORM_MACOS, PLATFORM_WINDOWS],
            on_device: true,
            requires_api_key: false,
            api_key_provider_id: None,
            featured: true,
        },
        SpeechProviderMetadata {
            id: "whisper_openai",
            display_name: "OpenAI Whisper",
            description: "Cloud transcription via OpenAI's Whisper API. \
                          Requires an OpenAI API key (shared with the OpenAI AI provider).",
            api_format: SpeechApiFormat::WhisperOpenAi,
            supported_platforms: &[PLATFORM_MACOS, PLATFORM_WINDOWS, PLATFORM_LINUX],
            on_device: false,
            requires_api_key: true,
            api_key_provider_id: Some("openai"),
            featured: false,
        },
    ]
}

pub fn find_speech_metadata(id: &str) -> Option<SpeechProviderMetadata> {
    speech_provider_catalog().into_iter().find(|m| m.id == id)
}

/// Default speech provider id for first launch — pick the system-native
/// recogniser when the platform supports it, otherwise fall back to Whisper.
pub fn default_speech_provider_id() -> &'static str {
    let system = SpeechProviderMetadata {
        id: "system",
        display_name: "",
        description: "",
        api_format: SpeechApiFormat::System,
        supported_platforms: &[PLATFORM_MACOS, PLATFORM_WINDOWS],
        on_device: true,
        requires_api_key: false,
        api_key_provider_id: None,
        featured: true,
    };
    if system.available_on_current_platform() {
        "system"
    } else {
        "whisper_openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_has_system_and_whisper() {
        let ids: Vec<_> = speech_provider_catalog()
            .into_iter()
            .map(|m| m.id)
            .collect();
        assert!(ids.contains(&"system"));
        assert!(ids.contains(&"whisper_openai"));
    }

    #[test]
    fn find_unknown_returns_none() {
        assert!(find_speech_metadata("does-not-exist").is_none());
    }

    #[test]
    fn whisper_is_available_on_every_platform() {
        let whisper = find_speech_metadata("whisper_openai").unwrap();
        assert!(whisper.supported_platforms.contains(&PLATFORM_MACOS));
        assert!(whisper.supported_platforms.contains(&PLATFORM_WINDOWS));
        assert!(whisper.supported_platforms.contains(&PLATFORM_LINUX));
    }

    #[test]
    fn default_provider_id_matches_current_platform() {
        let default_id = default_speech_provider_id();
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        assert_eq!(default_id, "system");
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        assert_eq!(default_id, "whisper_openai");
    }
}
