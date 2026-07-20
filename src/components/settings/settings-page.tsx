import { X } from "lucide-react";
import { useEffect, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { AboutPanel } from "@/components/settings/about-panel";
import { DisplayPanel } from "@/components/settings/display-panel";
import { GeneralPanel } from "@/components/settings/general-panel";
import { MemoryPanel } from "@/components/settings/memory-panel";
import { PlaceholderPanel } from "@/components/settings/placeholder-panel";
import { PrivacyPanel } from "@/components/settings/privacy-panel";
import { ProviderConfigPanel } from "@/components/settings/provider-config";
import { type SettingsSection, SettingsSidebar } from "@/components/settings/settings-sidebar";
import { ShortcutsPanel } from "@/components/settings/shortcuts-panel";
import { SoundsPanel } from "@/components/settings/sounds-panel";
import { SyncPanel } from "@/components/settings/sync-panel";
import { Button } from "@/components/ui/button";
import { strings } from "@/lib/i18n";
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

  const [selected, setSelected] = useState<SettingsSection | null>(null);

  useEffect(() => {
    if (!hydrated) {
      void hydrate();
    }
  }, [hydrated, hydrate]);

  useEffect(() => {
    if (!hydrated || selected !== null) return;
    setSelected(activeId ? { kind: "provider", providerId: activeId } : { kind: "general" });
  }, [hydrated, activeId, selected]);

  const current: SettingsSection = selected ?? { kind: "general" };

  return (
    <div className="flex h-screen bg-background">
      <SettingsSidebar
        providers={catalog}
        selected={current}
        activeId={activeId}
        hasCredentials={hasCredentials}
        onSelect={setSelected}
      />

      <div className="flex min-w-0 flex-1 flex-col">
        <header
          data-tauri-drag-region
          className="flex h-[52px] flex-shrink-0 select-none items-center justify-between border-b-[0.5px] border-border px-6"
        >
          <div className="pointer-events-none flex items-center gap-2.5">
            <AcornLogo size={20} />
            <div className="text-[15px] font-semibold text-foreground">Settings</div>
          </div>
          <Button variant="outline" size="icon-sm" onClick={onClose} aria-label="Close settings">
            <X />
          </Button>
        </header>

        <SettingsContent section={current} />
      </div>
    </div>
  );
}

function SettingsContent({ section }: { section: SettingsSection }) {
  const catalog = useProvidersStore((s) => s.catalog);
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  if (section.kind === "general") return <GeneralPanel />;
  if (section.kind === "shortcuts") return <ShortcutsPanel />;
  if (section.kind === "about") return <AboutPanel />;

  if (section.kind === "mascot") {
    return (
      <PlaceholderPanel
        title={t.mascotSection}
        description={
          language.startsWith("zh")
            ? "选一只桌面小伴侣,它会在你工作时陪着你。"
            : "Pick a small desktop companion that keeps you company while you work."
        }
      />
    );
  }

  if (section.kind === "display") return <DisplayPanel />;

  if (section.kind === "sounds") return <SoundsPanel />;

  if (section.kind === "privacy") return <PrivacyPanel />;

  if (section.kind === "memory") return <MemoryPanel />;

  if (section.kind === "sync") return <SyncPanel />;

  const provider = catalog.find((p) => p.id === section.providerId);
  if (!provider) {
    return (
      <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7 text-[13px] text-muted-foreground">
        Pick a section on the left.
      </section>
    );
  }
  return <ProviderConfigPanel provider={provider} />;
}
