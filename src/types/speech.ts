export type SpeechApiFormat = "system" | "whisper_open_ai";

export type SpeechProviderStatus = "available" | "coming_soon";

export interface SpeechProviderMetadata {
  id: string;
  displayName: string;
  description: string;
  apiFormat: SpeechApiFormat;
  supportedPlatforms: string[];
  onDevice: boolean;
  requiresApiKey: boolean;
  apiKeyProviderId: string | null;
  featured: boolean;
  status: SpeechProviderStatus;
}

export interface SpeechProviderConfig {
  providerId: string;
  enabled: boolean;
  selectedLanguage: string | null;
  lastUsedAt: string | null;
}

export interface TranscribeOutcome {
  text: string;
  providerId: string;
}
