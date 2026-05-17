import { X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { ProviderConfigPanel } from "@/components/settings/provider-config";
import { ProviderSidebar } from "@/components/settings/provider-sidebar";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { useProvidersStore } from "@/stores/providers";
import { useSettingsStore } from "@/stores/settings";

interface SettingsPageProps {
  onClose: () => void;
}

export function SettingsPage({ onClose }: SettingsPageProps) {
  const catalog = useProvidersStore((s) => s.catalog);
  const hasCredentials = useProvidersStore((s) => s.hasCredentials);
  const hydrate = useProvidersStore((s) => s.hydrate);
  const hydrated = useProvidersStore((s) => s.hydrated);

  const activeId = useSettingsStore((s) => s.activeProviderId);
  const theme = useSettingsStore((s) => s.theme);
  const language = useSettingsStore((s) => s.language);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const setLanguage = useSettingsStore((s) => s.setLanguage);

  const defaultSelection = useMemo(
    () => activeId ?? catalog.find((p) => p.status === "available")?.id ?? null,
    [activeId, catalog],
  );
  const [selectedId, setSelectedId] = useState<string | null>(defaultSelection);

  useEffect(() => {
    if (!hydrated) {
      void hydrate();
    }
  }, [hydrated, hydrate]);

  useEffect(() => {
    if (selectedId === null && defaultSelection) {
      setSelectedId(defaultSelection);
    }
  }, [defaultSelection, selectedId]);

  const selectedProvider = catalog.find((p) => p.id === selectedId) ?? null;

  return (
    <div className="min-h-screen bg-background">
      <header className="flex items-center justify-between px-7 py-4 border-b-[0.5px] border-border">
        <div className="flex items-center gap-2.5">
          <AcornLogo size={24} />
          <div className="text-base font-medium text-foreground">Settings</div>
        </div>
        <Button variant="outline" size="icon-sm" onClick={onClose} aria-label="Close settings">
          <X />
        </Button>
      </header>

      <div className="flex" style={{ minHeight: "calc(100vh - 65px)" }}>
        <ProviderSidebar
          providers={catalog}
          selectedId={selectedId}
          activeId={activeId}
          hasCredentials={hasCredentials}
          onSelect={setSelectedId}
        />

        {selectedProvider ? (
          <ProviderConfigPanel provider={selectedProvider} />
        ) : (
          <section className="flex-1 px-8 py-7 text-muted-foreground text-sm">
            Pick a provider on the left.
          </section>
        )}
      </div>

      <div className="border-t-[0.5px] border-border px-8 py-5 flex items-center gap-8">
        <div className="flex items-center gap-3">
          <Label htmlFor="theme-toggle" className="text-sm">
            Dark mode
          </Label>
          <Switch
            id="theme-toggle"
            checked={theme === "dark"}
            onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
          />
        </div>
        <div className="flex items-center gap-3">
          <Label htmlFor="lang-select" className="text-sm">
            Language
          </Label>
          <select
            id="lang-select"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            className="bg-card border-[0.5px] border-border rounded-md px-3 py-1.5 text-sm"
          >
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
        </div>
      </div>
    </div>
  );
}
