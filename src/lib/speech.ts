import { invoke } from "@tauri-apps/api/core";

import type {
  SpeechProviderConfig,
  SpeechProviderMetadata,
  TranscribeOutcome,
} from "@/types/speech";

export const speechProviders = {
  list: () => invoke<SpeechProviderMetadata[]>("list_speech_providers"),

  configs: () => invoke<SpeechProviderConfig[]>("list_speech_provider_configs"),

  saveConfig: (providerId: string, enabled: boolean, selectedLanguage: string | null) =>
    invoke<SpeechProviderConfig>("save_speech_provider_config", {
      providerId,
      enabled,
      selectedLanguage,
    }),

  getActive: () => invoke<string | null>("get_active_speech_provider"),

  setActive: (providerId: string) => invoke<void>("set_active_speech_provider", { providerId }),

  transcribe: (audio: Uint8Array, mimeType: string, language?: string) =>
    invoke<TranscribeOutcome>("transcribe_audio", {
      audio: Array.from(audio),
      mimeType,
      language: language ?? null,
    }),
};
