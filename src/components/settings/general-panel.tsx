import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

export function GeneralPanel() {
  const theme = useSettingsStore((s) => s.theme);
  const language = useSettingsStore((s) => s.language);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const setLanguage = useSettingsStore((s) => s.setLanguage);
  const t = strings(language);

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <h2 className="mb-1 text-[20px] font-semibold text-foreground">{t.generalPanelTitle}</h2>
      <p className="mb-6 text-[13px] text-muted-foreground">{t.generalPanelSubtitle}</p>

      <div className="max-w-xl divide-y-[0.5px] divide-border rounded-lg border-[0.5px] border-border bg-card shadow-[var(--shadow-card)]">
        <Row label={t.themeLabel} caption={t.themeCaption} htmlFor="theme-toggle">
          <Switch
            id="theme-toggle"
            checked={theme === "dark"}
            onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
          />
        </Row>

        <Row label={t.languageLabel} caption={t.languageCaption} htmlFor="lang-select">
          <select
            id="lang-select"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            className="rounded-md border-[0.5px] border-border bg-background px-2.5 py-1 text-[13px]"
          >
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
        </Row>
      </div>
    </section>
  );
}

interface RowProps {
  label: string;
  caption: string;
  htmlFor: string;
  children: React.ReactNode;
}

function Row({ label, caption, htmlFor, children }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-6 px-4 py-3">
      <div className="min-w-0">
        <Label htmlFor={htmlFor} className="text-[13px] text-foreground">
          {label}
        </Label>
        <div className="mt-0.5 text-[11px] text-muted-foreground">{caption}</div>
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}
