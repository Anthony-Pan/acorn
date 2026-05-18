import { Channel, invoke } from "@tauri-apps/api/core";

import { speechProviders } from "@/lib/speech";
import type {
  DecomposeEvent,
  DecomposeRequest,
  DecomposeResponse,
  ProviderMetadata,
} from "@/types/ai";

export const providersCatalog = () => invoke<ProviderMetadata[]>("list_providers");

export const credentials = {
  save: (providerId: string, apiKey: string) =>
    invoke<void>("save_provider_credentials", { providerId, apiKey }),

  delete: (providerId: string) => invoke<void>("delete_provider_credentials", { providerId }),

  has: (providerId: string) => invoke<boolean>("has_provider_credentials", { providerId }),

  test: (providerId: string) => invoke<void>("test_provider_connection", { providerId }),
};

export function decompose(
  providerId: string,
  request: DecomposeRequest,
  onEvent: (event: DecomposeEvent) => void,
): Promise<DecomposeResponse> {
  const channel = new Channel<DecomposeEvent>();
  channel.onmessage = onEvent;
  return invoke<DecomposeResponse>("decompose", {
    providerId,
    request,
    onEvent: channel,
  });
}

export async function transcribe(
  audio: Uint8Array,
  mimeType: string,
  language?: string,
): Promise<string> {
  const outcome = await speechProviders.transcribe(audio, mimeType, language);
  return outcome.text;
}
