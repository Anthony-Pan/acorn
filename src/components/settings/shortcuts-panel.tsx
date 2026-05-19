import { ShortcutRecorder } from "@/components/settings/shortcut-recorder";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

export function ShortcutsPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  return (
    <section className="flex-1 px-8 py-7">
      <h2 className="text-[20px] font-medium text-foreground mb-1">{t.shortcutsPanelTitle}</h2>
      <p className="text-[13px] text-muted-foreground mb-6">{t.shortcutsPanelSubtitle}</p>

      <div className="space-y-5 max-w-xl">
        <div className="py-3 border-b-[0.5px] border-border/60">
          <ShortcutRecorder language={language} />
        </div>
      </div>
    </section>
  );
}
