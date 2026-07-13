import { ShortcutRecorder } from "@/components/settings/shortcut-recorder";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

export function ShortcutsPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <h2 className="mb-1 text-[20px] font-semibold text-foreground">{t.shortcutsPanelTitle}</h2>
      <p className="mb-6 text-[13px] text-muted-foreground">{t.shortcutsPanelSubtitle}</p>

      <div className="max-w-xl rounded-lg border-[0.5px] border-border bg-card shadow-[var(--shadow-card)]">
        <div className="px-4 py-3">
          <ShortcutRecorder language={language} />
        </div>
      </div>
    </section>
  );
}
