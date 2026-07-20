import {
  Bell,
  Check,
  Cookie,
  Info,
  Keyboard,
  Monitor,
  RefreshCw,
  Settings as SettingsIcon,
  ShieldCheck,
  Sparkles,
  Sprout,
} from "lucide-react";

import { strings } from "@/lib/i18n";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/stores/settings";
import type { ProviderCategory, ProviderMetadata } from "@/types/ai";

export type SettingsSection =
  | { kind: "general" }
  | { kind: "shortcuts" }
  | { kind: "mascot" }
  | { kind: "display" }
  | { kind: "sounds" }
  | { kind: "privacy" }
  | { kind: "memory" }
  | { kind: "sync" }
  | { kind: "about" }
  | { kind: "provider"; providerId: string };

export function sectionKey(section: SettingsSection): string {
  return section.kind === "provider" ? `provider:${section.providerId}` : section.kind;
}

interface SettingsSidebarProps {
  providers: ProviderMetadata[];
  selected: SettingsSection;
  activeId: string | null;
  hasCredentials: Record<string, boolean>;
  onSelect: (section: SettingsSection) => void;
}

const CATEGORY_ORDER: ProviderCategory[] = ["recommended", "local", "advanced", "coming_soon"];

export function SettingsSidebar({
  providers,
  selected,
  activeId,
  hasCredentials,
  onSelect,
}: SettingsSidebarProps) {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);
  const categoryLabel: Record<ProviderCategory, string> = {
    recommended: t.categoryRecommended,
    local: t.categoryLocal,
    advanced: t.categoryAdvanced,
    coming_soon: t.categoryComingSoon,
  };
  const grouped = group(providers);
  const selectedKey = sectionKey(selected);

  return (
    <nav className="flex w-[200px] flex-shrink-0 flex-col border-r-[0.5px] border-border bg-sidebar">
      <div data-tauri-drag-region className="h-[36px] flex-shrink-0" />
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto px-2 pb-2">
        <div className="mb-4">
          <GroupLabel>{t.settingsGroup}</GroupLabel>
          <SectionRow
            icon={<SettingsIcon className="h-4 w-4" />}
            label={t.generalSection}
            selected={selectedKey === "general"}
            onClick={() => onSelect({ kind: "general" })}
          />
          <SectionRow
            icon={<Keyboard className="h-4 w-4" />}
            label={t.shortcutsSection}
            selected={selectedKey === "shortcuts"}
            onClick={() => onSelect({ kind: "shortcuts" })}
          />
          <SectionRow
            icon={<Sprout className="h-4 w-4" />}
            label={t.mascotSection}
            selected={selectedKey === "mascot"}
            onClick={() => onSelect({ kind: "mascot" })}
            comingSoonLabel={t.comingSoonBadge}
          />
          <SectionRow
            icon={<Monitor className="h-4 w-4" />}
            label={t.displaySection}
            selected={selectedKey === "display"}
            onClick={() => onSelect({ kind: "display" })}
          />
          <SectionRow
            icon={<Bell className="h-4 w-4" />}
            label={t.soundsSection}
            selected={selectedKey === "sounds"}
            onClick={() => onSelect({ kind: "sounds" })}
          />
          <SectionRow
            icon={<Cookie className="h-4 w-4" />}
            label={t.privacySection}
            selected={selectedKey === "privacy"}
            onClick={() => onSelect({ kind: "privacy" })}
          />
          <SectionRow
            icon={<ShieldCheck className="h-4 w-4" />}
            label={t.memorySection}
            selected={selectedKey === "memory"}
            onClick={() => onSelect({ kind: "memory" })}
          />
          <SectionRow
            icon={<RefreshCw className="h-4 w-4" />}
            label={t.syncSection}
            selected={selectedKey === "sync"}
            onClick={() => onSelect({ kind: "sync" })}
          />
          <SectionRow
            icon={<Info className="h-4 w-4" />}
            label={t.aboutSection}
            selected={selectedKey === "about"}
            onClick={() => onSelect({ kind: "about" })}
          />
        </div>

        <GroupLabel>{t.modelsGroup}</GroupLabel>
        {CATEGORY_ORDER.map((category) => {
          const items = grouped[category];
          if (!items || items.length === 0) return null;
          return (
            <div key={category} className="mb-4">
              <SubGroupLabel>{categoryLabel[category]}</SubGroupLabel>
              <div className="space-y-px">
                {items.map((p) => (
                  <ProviderRow
                    key={p.id}
                    provider={p}
                    selected={selectedKey === `provider:${p.id}`}
                    active={activeId === p.id}
                    hasKey={hasCredentials[p.id] ?? !p.requiresApiKey}
                    onSelect={(id) => onSelect({ kind: "provider", providerId: id })}
                  />
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </nav>
  );
}

function GroupLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="mb-1 select-none px-2 text-[11px] font-semibold tracking-wide text-muted-foreground">
      {children}
    </div>
  );
}

function SubGroupLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="mb-1 select-none px-2 text-[11px] font-medium tracking-wide text-muted-foreground/80">
      {children}
    </div>
  );
}

interface SectionRowProps {
  icon: React.ReactNode;
  label: string;
  selected: boolean;
  onClick: () => void;
  comingSoonLabel?: string;
}

function SectionRow({ icon, label, selected, onClick, comingSoonLabel }: SectionRowProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex w-full select-none items-center gap-2 rounded-md px-2 py-[5px] text-left text-[13px] transition-colors",
        selected
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/85 hover:bg-sidebar-accent/50",
      )}
    >
      <span className="flex-shrink-0 text-acorn-orange/80">{icon}</span>
      <span className="flex-1 truncate">{label}</span>
      {comingSoonLabel ? (
        <span className="rounded bg-muted/60 px-1 py-px font-mono text-[9px] uppercase tracking-wider text-muted-foreground/70">
          {comingSoonLabel}
        </span>
      ) : null}
    </button>
  );
}

interface ProviderRowProps {
  provider: ProviderMetadata;
  selected: boolean;
  active: boolean;
  hasKey: boolean;
  onSelect: (id: string) => void;
}

function ProviderRow({ provider, selected, active, hasKey, onSelect }: ProviderRowProps) {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);
  const comingSoon = provider.status === "coming_soon";
  return (
    <button
      type="button"
      onClick={() => onSelect(provider.id)}
      disabled={comingSoon}
      className={cn(
        "flex w-full select-none items-center gap-2 rounded-md px-2 py-[5px] text-left text-[13px] transition-colors",
        selected
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/85 hover:bg-sidebar-accent/50",
        comingSoon && "cursor-not-allowed text-muted-foreground/50",
      )}
    >
      {provider.featured ? (
        <Sparkles className="h-4 w-4 flex-shrink-0 text-acorn-orange" />
      ) : (
        <span className="w-4" />
      )}
      <span className="flex-1 truncate">{provider.displayName}</span>
      {!comingSoon && active ? (
        <span className="text-[11px] font-medium text-acorn-orange">{t.activeBadge}</span>
      ) : null}
      {!comingSoon && hasKey && !active ? <Check className="h-3.5 w-3.5 text-acorn-olive" /> : null}
    </button>
  );
}

function group(providers: ProviderMetadata[]): Record<ProviderCategory, ProviderMetadata[]> {
  const out: Record<ProviderCategory, ProviderMetadata[]> = {
    recommended: [],
    local: [],
    advanced: [],
    coming_soon: [],
  };
  for (const p of providers) out[p.category].push(p);
  return out;
}
