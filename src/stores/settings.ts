import { create } from "zustand";

import { providers, settings } from "@/lib/db";

type Theme = "light" | "dark";

interface SettingsState {
  activeProviderId: string | null;
  theme: Theme;
  language: string;
  hydrated: boolean;

  hydrate: () => Promise<void>;
  setActiveProvider: (providerId: string) => Promise<void>;
  setTheme: (theme: Theme) => Promise<void>;
  setLanguage: (language: string) => Promise<void>;
}

const THEME_KEY = "theme";
const LANGUAGE_KEY = "language";

function detectLanguage(): string {
  const nav = typeof navigator !== "undefined" ? navigator.language : "en";
  return nav.startsWith("zh") ? "zh" : "en";
}

export const useSettingsStore = create<SettingsState>((set) => ({
  activeProviderId: null,
  theme: "light",
  language: detectLanguage(),
  hydrated: false,

  hydrate: async () => {
    const [active, theme, lang] = await Promise.all([
      providers.getActive(),
      settings.get(THEME_KEY),
      settings.get(LANGUAGE_KEY),
    ]);

    set({
      activeProviderId: active,
      theme: (theme as Theme) ?? "light",
      language: lang ?? detectLanguage(),
      hydrated: true,
    });

    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", theme === "dark");
    }
  },

  setActiveProvider: async (providerId) => {
    await providers.setActive(providerId);
    set({ activeProviderId: providerId });
  },

  setTheme: async (theme) => {
    await settings.set(THEME_KEY, theme);
    set({ theme });
    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", theme === "dark");
    }
  },

  setLanguage: async (language) => {
    await settings.set(LANGUAGE_KEY, language);
    set({ language });
  },
}));
