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
    <section className="flex-1 px-8 py-7">
      <h2 className="text-[20px] font-medium text-foreground mb-1">{t.generalPanelTitle}</h2>
      <p className="text-[13px] text-muted-foreground mb-6">{t.generalPanelSubtitle}</p>

      <div className="space-y-5 max-w-xl">
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
            className="bg-card border-[0.5px] border-border rounded-md px-3 py-1.5 text-sm"
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
    <div className="flex items-start justify-between gap-6 py-3 border-b-[0.5px] border-border/60">
      <div className="min-w-0">
        <Label htmlFor={htmlFor} className="text-sm text-foreground">
          {label}
        </Label>
        <div className="text-[12px] text-muted-foreground mt-0.5">{caption}</div>
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}
