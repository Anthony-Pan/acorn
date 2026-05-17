import { create } from "zustand";

import { credentials, providersCatalog } from "@/lib/ai";
import { providers as providerConfigs } from "@/lib/db";
import type { ProviderMetadata } from "@/types/ai";
import type { ProviderConfig } from "@/types/db";

interface ProvidersState {
  catalog: ProviderMetadata[];
  configs: Record<string, ProviderConfig>;
  hasCredentials: Record<string, boolean>;
  hydrated: boolean;

  hydrate: () => Promise<void>;
  saveCredentials: (providerId: string, apiKey: string) => Promise<void>;
  deleteCredentials: (providerId: string) => Promise<void>;
  saveConfig: (
    providerId: string,
    patch: Partial<Pick<ProviderConfig, "enabled" | "customEndpoint" | "selectedModel">>,
  ) => Promise<void>;
  testConnection: (providerId: string) => Promise<void>;
}

export const useProvidersStore = create<ProvidersState>((set, get) => ({
  catalog: [],
  configs: {},
  hasCredentials: {},
  hydrated: false,

  hydrate: async () => {
    const [catalog, configList] = await Promise.all([providersCatalog(), providerConfigs.list()]);

    const configs: Record<string, ProviderConfig> = {};
    for (const c of configList) {
      configs[c.providerId] = c;
    }

    const presence = await Promise.all(
      catalog
        .filter((m) => m.requiresApiKey)
        .map(async (m) => [m.id, await credentials.has(m.id)] as const),
    );
    const hasCredentials: Record<string, boolean> = {};
    for (const [id, present] of presence) {
      hasCredentials[id] = present;
    }

    set({ catalog, configs, hasCredentials, hydrated: true });
  },

  saveCredentials: async (providerId, apiKey) => {
    await credentials.save(providerId, apiKey);
    set((s) => ({ hasCredentials: { ...s.hasCredentials, [providerId]: true } }));
  },

  deleteCredentials: async (providerId) => {
    await credentials.delete(providerId);
    set((s) => ({ hasCredentials: { ...s.hasCredentials, [providerId]: false } }));
  },

  saveConfig: async (providerId, patch) => {
    const existing = get().configs[providerId];
    const enabled = patch.enabled ?? existing?.enabled ?? true;
    const customEndpoint = patch.customEndpoint ?? existing?.customEndpoint ?? null;
    const selectedModel = patch.selectedModel ?? existing?.selectedModel ?? null;

    const saved = await providerConfigs.save(providerId, enabled, customEndpoint, selectedModel);
    set((s) => ({ configs: { ...s.configs, [providerId]: saved } }));
  },

  testConnection: async (providerId) => {
    await credentials.test(providerId);
  },
}));
