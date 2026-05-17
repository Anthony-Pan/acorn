import { Check, Sparkles } from "lucide-react";

import { cn } from "@/lib/utils";
import type { ProviderCategory, ProviderMetadata } from "@/types/ai";

interface ProviderSidebarProps {
  providers: ProviderMetadata[];
  selectedId: string | null;
  activeId: string | null;
  hasCredentials: Record<string, boolean>;
  onSelect: (id: string) => void;
}

const CATEGORY_LABEL: Record<ProviderCategory, string> = {
  recommended: "Recommended",
  local: "Local",
  advanced: "Advanced",
  coming_soon: "Coming soon",
};

const CATEGORY_ORDER: ProviderCategory[] = ["recommended", "local", "advanced", "coming_soon"];

export function ProviderSidebar({
  providers,
  selectedId,
  activeId,
  hasCredentials,
  onSelect,
}: ProviderSidebarProps) {
  const grouped = group(providers);

  return (
    <nav className="w-56 flex-shrink-0 border-r-[0.5px] border-border overflow-y-auto py-4">
      {CATEGORY_ORDER.map((category) => {
        const items = grouped[category];
        if (!items || items.length === 0) return null;
        return (
          <div key={category} className="mb-4">
            <div className="px-4 mb-1.5 text-[10px] uppercase tracking-wider text-muted-foreground font-medium">
              {CATEGORY_LABEL[category]}
            </div>
            <div>
              {items.map((p) => (
                <ProviderRow
                  key={p.id}
                  provider={p}
                  selected={selectedId === p.id}
                  active={activeId === p.id}
                  hasKey={hasCredentials[p.id] ?? !p.requiresApiKey}
                  onSelect={onSelect}
                />
              ))}
            </div>
          </div>
        );
      })}
    </nav>
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
        <span className="text-[10px] text-acorn-orange font-medium">ACTIVE</span>
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
