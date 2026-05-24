import {
  Bell,
  Check,
  Cookie,
  Info,
  Keyboard,
  Monitor,
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
    <nav className="w-56 flex-shrink-0 border-r-[0.5px] border-border overflow-y-auto py-4">
      <div className="mb-4">
        <GroupLabel>{t.settingsGroup}</GroupLabel>
        <SectionRow
          icon={<SettingsIcon className="w-3.5 h-3.5" />}
          label={t.generalSection}
          selected={selectedKey === "general"}
          onClick={() => onSelect({ kind: "general" })}
        />
        <SectionRow
          icon={<Keyboard className="w-3.5 h-3.5" />}
          label={t.shortcutsSection}
          selected={selectedKey === "shortcuts"}
          onClick={() => onSelect({ kind: "shortcuts" })}
        />
        <SectionRow
          icon={<Sprout className="w-3.5 h-3.5" />}
          label={t.mascotSection}
          selected={selectedKey === "mascot"}
          onClick={() => onSelect({ kind: "mascot" })}
          comingSoonLabel={t.comingSoonBadge}
        />
        <SectionRow
          icon={<Monitor className="w-3.5 h-3.5" />}
          label={t.displaySection}
          selected={selectedKey === "display"}
          onClick={() => onSelect({ kind: "display" })}
          comingSoonLabel={t.comingSoonBadge}
        />
        <SectionRow
          icon={<Bell className="w-3.5 h-3.5" />}
          label={t.soundsSection}
          selected={selectedKey === "sounds"}
          onClick={() => onSelect({ kind: "sounds" })}
        />
        <SectionRow
          icon={<Cookie className="w-3.5 h-3.5" />}
          label={t.privacySection}
          selected={selectedKey === "privacy"}
          onClick={() => onSelect({ kind: "privacy" })}
          comingSoonLabel={t.comingSoonBadge}
        />
        <SectionRow
          icon={<ShieldCheck className="w-3.5 h-3.5" />}
          label={t.memorySection}
          selected={selectedKey === "memory"}
          onClick={() => onSelect({ kind: "memory" })}
          comingSoonLabel={t.comingSoonBadge}
        />
        <SectionRow
          icon={<Info className="w-3.5 h-3.5" />}
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
            <div>
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
    </nav>
  );
}

function GroupLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="px-4 mb-1.5 text-[10px] uppercase tracking-wider text-acorn-brown-deep font-semibold">
      {children}
    </div>
  );
}

function SubGroupLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="px-4 mb-1.5 text-[10px] uppercase tracking-wider text-muted-foreground font-medium">
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
        "w-full text-left px-4 py-2 text-[13px] flex items-center gap-2 transition-colors",
        selected ? "bg-acorn-orange/12 text-foreground" : "text-foreground/85 hover:bg-muted",
      )}
    >
      <span className="text-acorn-orange/80 flex-shrink-0">{icon}</span>
      <span className="flex-1 truncate">{label}</span>
      {comingSoonLabel ? (
        <span className="text-[9px] font-mono uppercase tracking-wider text-muted-foreground/70 px-1 py-px rounded bg-muted/60">
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
        "w-full text-left px-4 py-2 text-[13px] flex items-center gap-2 transition-colors",
        selected ? "bg-acorn-orange/12 text-foreground" : "text-foreground/85 hover:bg-muted",
        comingSoon && "text-muted-foreground/50 cursor-not-allowed",
      )}
    >
      {provider.featured ? (
        <Sparkles className="w-3 h-3 text-acorn-orange flex-shrink-0" />
      ) : (
        <span className="w-3" />
      )}
      <span className="flex-1 truncate">{provider.displayName}</span>
      {!comingSoon && active ? (
        <span className="text-[10px] text-acorn-orange font-medium">{t.activeBadge}</span>
      ) : null}
      {!comingSoon && hasKey && !active ? <Check className="w-3 h-3 text-acorn-olive" /> : null}
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
