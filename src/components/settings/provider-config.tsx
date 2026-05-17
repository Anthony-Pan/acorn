import { Check, ExternalLink, Eye, EyeOff, Loader2 } from "lucide-react";
import { useEffect, useState } from "react";

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

  useEffect(() => {
    setApiKey("");
    setShowKey(false);
    setSelectedModel(config?.selectedModel ?? provider.defaultModel);
    setEndpoint(config?.customEndpoint ?? "");
    setTest({ status: "idle" });
  }, [provider, config]);

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
      <section className="flex-1 px-8 py-7 overflow-y-auto">
        <h2 className="text-xl font-medium text-foreground mb-1.5">{provider.displayName}</h2>
        <p className="text-sm text-muted-foreground">
          Coming soon. Acorn Cloud will handle model routing, key management, and billing for you.
        </p>
      </section>
    );
  }

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <div className="flex items-start justify-between mb-6">
        <div>
          <h2 className="text-xl font-medium text-foreground">{provider.displayName}</h2>
          <p className="text-xs text-muted-foreground mt-1">
            {provider.featured ? "Recommended · " : null}
            {provider.requiresApiKey ? "API key required" : "Runs locally — no API key needed"}
          </p>
        </div>
        {!isActive ? (
          <Button onClick={handleMakeActive} variant="outline" size="sm">
            Make active
          </Button>
        ) : (
          <span className="text-[11px] text-acorn-orange font-medium uppercase tracking-wider">
            Currently active
          </span>
        )}
      </div>

      {provider.requiresApiKey ? (
        <div className="mb-6">
          <Label className="text-xs text-muted-foreground mb-1.5 block">API Key</Label>
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

      <div className="mb-5">
        <Label className="text-xs text-muted-foreground mb-1.5 block">Model</Label>
        <select
          value={selectedModel}
          onChange={(e) => setSelectedModel(e.target.value)}
          onBlur={handlePersistConfig}
          className="w-full bg-card border-[0.5px] border-border rounded-md px-3 py-2 text-sm"
        >
          {provider.availableModels.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>
      </div>

      {provider.allowCustomEndpoint ? (
        <div className="mb-6">
          <Label className="text-xs text-muted-foreground mb-1.5 block">
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

      <div className="pt-3 border-t-[0.5px] border-border flex items-center gap-3">
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
          <span className="inline-flex items-center gap-1.5 text-xs text-acorn-olive">
            <Check className="w-3.5 h-3.5" /> Connection works
          </span>
        ) : null}
        {test.status === "error" ? (
          <span className="text-xs text-acorn-red truncate max-w-[280px]" title={test.message}>
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
