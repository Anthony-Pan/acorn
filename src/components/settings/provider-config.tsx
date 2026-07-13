import { invoke } from "@tauri-apps/api/core";
import { Check, ExternalLink, Eye, EyeOff, Loader2, RotateCcw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useProvidersStore } from "@/stores/providers";
import { useSettingsStore } from "@/stores/settings";
import type { ProviderMetadata } from "@/types/ai";

type TestState =
  | { status: "idle" }
  | { status: "testing" }
  | { status: "ok" }
  | { status: "error"; message: string };

interface ProviderConfigPanelProps {
  provider: ProviderMetadata;
}

export function ProviderConfigPanel({ provider }: ProviderConfigPanelProps) {
  const config = useProvidersStore((s) => s.configs[provider.id]);
  const hasKey = useProvidersStore((s) => s.hasCredentials[provider.id]);
  const activeId = useSettingsStore((s) => s.activeProviderId);
  const setActiveProvider = useSettingsStore((s) => s.setActiveProvider);
  const saveCredentials = useProvidersStore((s) => s.saveCredentials);
  const deleteCredentials = useProvidersStore((s) => s.deleteCredentials);
  const saveConfig = useProvidersStore((s) => s.saveConfig);
  const testConnection = useProvidersStore((s) => s.testConnection);

  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [selectedModel, setSelectedModel] = useState(
    config?.selectedModel ?? provider.defaultModel,
  );
  const [endpoint, setEndpoint] = useState(config?.customEndpoint ?? "");
  const [test, setTest] = useState<TestState>({ status: "idle" });
  const [ollamaModels, setOllamaModels] = useState<string[] | null>(null);
  const [ollamaError, setOllamaError] = useState<string | null>(null);
  const [refreshingOllama, setRefreshingOllama] = useState(false);

  const refreshOllamaModels = useCallback(
    async (customEndpoint: string | null) => {
      if (provider.id !== "ollama") return;
      setRefreshingOllama(true);
      setOllamaError(null);
      try {
        const models = await invoke<string[]>("list_ollama_models", {
          customEndpoint: customEndpoint || null,
        });
        setOllamaModels(models);
      } catch (err) {
        setOllamaModels([]);
        setOllamaError(err instanceof Error ? err.message : String(err));
      } finally {
        setRefreshingOllama(false);
      }
    },
    [provider.id],
  );

  useEffect(() => {
    setApiKey("");
    setShowKey(false);
    setSelectedModel(config?.selectedModel ?? provider.defaultModel);
    setEndpoint(config?.customEndpoint ?? "");
    setTest({ status: "idle" });
    setOllamaModels(null);
    setOllamaError(null);
    if (provider.id === "ollama") {
      void refreshOllamaModels(config?.customEndpoint ?? null);
    }
  }, [provider, config, refreshOllamaModels]);

  const isActive = activeId === provider.id;
  const isComingSoon = provider.status === "coming_soon";

  const handleSaveKey = async () => {
    const trimmed = apiKey.trim();
    if (!trimmed) return;
    await saveCredentials(provider.id, trimmed);
    setApiKey("");
    setShowKey(false);
  };

  const handleDeleteKey = async () => {
    await deleteCredentials(provider.id);
  };

  const handlePersistConfig = async () => {
    await saveConfig(provider.id, {
      selectedModel,
      customEndpoint: endpoint.trim() || null,
      enabled: true,
    });
  };

  const handleTest = async () => {
    setTest({ status: "testing" });
    try {
      await testConnection(provider.id);
      setTest({ status: "ok" });
    } catch (err) {
      setTest({
        status: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  };

  const handleMakeActive = async () => {
    await setActiveProvider(provider.id);
  };

  if (isComingSoon) {
    return (
      <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
        <h2 className="mb-1.5 text-[20px] font-semibold text-foreground">{provider.displayName}</h2>
        <p className="text-[13px] text-muted-foreground">
          Coming soon. Acorn Cloud will handle model routing, key management, and billing for you.
        </p>
      </section>
    );
  }

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <div className="mb-6 flex max-w-2xl items-start justify-between">
        <div>
          <h2 className="text-[20px] font-semibold text-foreground">{provider.displayName}</h2>
          <p className="mt-0.5 text-[13px] text-muted-foreground">
            {provider.featured ? "Recommended · " : null}
            {provider.requiresApiKey ? "API key required" : "Runs locally — no API key needed"}
          </p>
        </div>
        {!isActive ? (
          <Button onClick={handleMakeActive} variant="outline" size="sm">
            Make active
          </Button>
        ) : (
          <span className="text-[11px] font-medium uppercase tracking-wider text-acorn-orange">
            Currently active
          </span>
        )}
      </div>

      <div className="max-w-2xl divide-y-[0.5px] divide-border rounded-lg border-[0.5px] border-border bg-card px-4 shadow-[var(--shadow-card)]">
        {provider.requiresApiKey ? (
          <div className="py-4">
            <Label className="mb-1.5 block text-[13px] text-foreground">API Key</Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Input
                  type={showKey ? "text" : "password"}
                  placeholder={hasKey ? "•••••• stored in keychain" : "Paste your key…"}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  className="pr-9"
                />
                <button
                  type="button"
                  onClick={() => setShowKey((s) => !s)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  aria-label={showKey ? "Hide" : "Show"}
                >
                  {showKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                </button>
              </div>
              <Button onClick={handleSaveKey} disabled={apiKey.trim().length === 0} size="sm">
                Save
              </Button>
              {hasKey ? (
                <Button onClick={handleDeleteKey} variant="outline" size="sm">
                  Forget
                </Button>
              ) : null}
            </div>
            {provider.apiKeyUrl ? (
              <a
                href={provider.apiKeyUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-1.5 inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground"
              >
                Get your key at {hostnameOf(provider.apiKeyUrl)}
                <ExternalLink className="w-3 h-3" />
              </a>
            ) : null}
          </div>
        ) : null}

        <div className="py-4">
          <div className="mb-1.5 flex items-center justify-between">
            <Label className="text-[13px] text-foreground">Model</Label>
            {provider.id === "ollama" ? (
              <button
                type="button"
                onClick={() => void refreshOllamaModels(endpoint || null)}
                disabled={refreshingOllama}
                className="inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground"
              >
                {refreshingOllama ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <RotateCcw className="w-3 h-3" />
                )}
                Refresh
              </button>
            ) : null}
          </div>
          <select
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value)}
            onBlur={handlePersistConfig}
            className="w-full rounded-md border-[0.5px] border-border bg-background px-3 py-1.5 text-[13px]"
          >
            {(provider.id === "ollama" && ollamaModels !== null
              ? ollamaModels.length > 0
                ? ollamaModels
                : provider.availableModels
              : provider.availableModels
            ).map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
          {provider.id === "ollama" && ollamaModels !== null && ollamaModels.length === 0 ? (
            <div className="mt-1.5 text-[11px] leading-relaxed text-muted-foreground">
              No models pulled yet. In a terminal, run:{" "}
              <code className="rounded bg-muted px-1 font-mono">ollama pull qwen2.5:7b</code> and
              click Refresh.
            </div>
          ) : null}
          {provider.id === "ollama" && ollamaError ? (
            <div className="mt-1.5 text-[11px] leading-relaxed text-acorn-red">{ollamaError}</div>
          ) : null}
        </div>

        {provider.allowCustomEndpoint ? (
          <div className="py-4">
            <Label className="mb-1.5 block text-[13px] text-foreground">
              Custom endpoint (optional)
            </Label>
            <Input
              value={endpoint}
              onChange={(e) => setEndpoint(e.target.value)}
              onBlur={handlePersistConfig}
              placeholder={provider.defaultEndpoint}
              spellCheck={false}
            />
          </div>
        ) : null}
      </div>

      <div className="mt-4 flex max-w-2xl items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          onClick={handleTest}
          disabled={test.status === "testing"}
        >
          {test.status === "testing" ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : null}
          Test connection
        </Button>
        {test.status === "ok" ? (
          <span className="inline-flex items-center gap-1.5 text-[13px] text-acorn-olive">
            <Check className="w-3.5 h-3.5" /> Connection works
          </span>
        ) : null}
        {test.status === "error" ? (
          <span className="max-w-[280px] truncate text-[13px] text-acorn-red" title={test.message}>
            {test.message}
          </span>
        ) : null}
      </div>
    </section>
  );
}

function hostnameOf(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}
