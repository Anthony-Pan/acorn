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
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <header className="mb-6">
        <div className="mb-2 flex items-center gap-2">
          <h1 className="text-[20px] font-semibold text-foreground">{title}</h1>
          <span className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
            {t.comingSoonBadge}
          </span>
        </div>
        <p className="text-[13px] text-muted-foreground">{description}</p>
      </header>

      <div className="max-w-md rounded-lg border-[0.5px] border-dashed border-border p-8 text-center">
        <Sprout className="mx-auto mb-3 h-6 w-6 text-acorn-orange/60" />
        <p className="text-[13px] text-muted-foreground">
          {language.startsWith("zh")
            ? "这部分正在路上,下次版本会落地。"
            : "Sprouting soon — landing in a future Acorn release."}
        </p>
      </div>
    </section>
  );
}
