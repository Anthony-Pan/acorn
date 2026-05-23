import { X } from "lucide-react";
import { useEffect, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { AboutPanel } from "@/components/settings/about-panel";
import { GeneralPanel } from "@/components/settings/general-panel";
import { MemoryPanel } from "@/components/settings/memory-panel";
import { PlaceholderPanel } from "@/components/settings/placeholder-panel";
import { ProviderConfigPanel } from "@/components/settings/provider-config";
import { type SettingsSection, SettingsSidebar } from "@/components/settings/settings-sidebar";
import { ShortcutsPanel } from "@/components/settings/shortcuts-panel";
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
        <SettingsSidebar
          providers={catalog}
          selected={current}
          activeId={activeId}
          hasCredentials={hasCredentials}
          onSelect={setSelected}
        />

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

  if (section.kind === "display") {
    return (
      <PlaceholderPanel
        title={t.displaySection}
        description={
          language.startsWith("zh")
            ? "窗口形态 / 顶栏胶囊 / Dock 图标可见性。"
            : "Window shape, top-bar capsule, and Dock icon visibility."
        }
      />
    );
  }

  if (section.kind === "sounds") {
    return (
      <PlaceholderPanel
        title={t.soundsSection}
        description={
          language.startsWith("zh")
            ? "为每个事件挑一个声音,或者干脆静音。"
            : "Pick a sound for each event, or mute the whole thing."
        }
      />
    );
  }

  if (section.kind === "privacy") {
    return (
      <PlaceholderPanel
        title={t.privacySection}
        description={
          language.startsWith("zh")
            ? "活动采集 · 黑名单 · 一键导出 / 清空。"
            : "Activity capture, blocklists, and one-tap export or wipe."
        }
      />
    );
  }

  if (section.kind === "memory") return <MemoryPanel />;

  const provider = catalog.find((p) => p.id === section.providerId);
  if (!provider) {
    return (
      <section className="flex-1 px-8 py-7 text-muted-foreground text-sm">
        Pick a section on the left.
      </section>
    );
  }
  return <ProviderConfigPanel provider={provider} />;
}
