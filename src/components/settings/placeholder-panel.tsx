import { Sprout } from "lucide-react";

import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

interface PlaceholderPanelProps {
  title: string;
  description: string;
}

export function PlaceholderPanel({ title, description }: PlaceholderPanelProps) {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-6">
        <div className="flex items-center gap-2 mb-2">
          <h1 className="text-lg font-medium text-foreground">{title}</h1>
          <span className="text-[10px] font-mono uppercase tracking-wider text-muted-foreground px-1.5 py-0.5 rounded bg-muted">
            {t.comingSoonBadge}
          </span>
        </div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </header>

      <div className="border-[0.5px] border-dashed border-border rounded-md p-8 text-center max-w-md">
        <Sprout className="w-6 h-6 text-acorn-orange/60 mx-auto mb-3" />
        <p className="text-sm text-muted-foreground">
          {language.startsWith("zh")
            ? "这部分正在路上,下次版本会落地。"
            : "Sprouting soon — landing in a future Acorn release."}
        </p>
      </div>
    </section>
  );
}
