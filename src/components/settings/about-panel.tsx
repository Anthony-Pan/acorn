import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { Check, Copy, ExternalLink } from "lucide-react";
import { useEffect, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

const REPO_URL = "https://github.com/onyxcraft/acorn";

export function AboutPanel() {
  const language = useSettingsStore((s) => s.language);
  const zh = language.startsWith("zh");
  const t = strings(language);

  const [appName, setAppName] = useState<string>("Acorn");
  const [version, setVersion] = useState<string>("");
  const [tauriVersion, setTauriVersion] = useState<string>("");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    void getName().then(setAppName);
    void getVersion().then(setVersion);
    void getTauriVersion().then(setTauriVersion);
  }, []);

  const buildMode = import.meta.env.MODE === "production" ? "release" : "dev";

  const copyDiagnostics = () => {
    const diagnostics = [
      `${appName} ${version || "?"} (${buildMode})`,
      `Tauri: ${tauriVersion || "?"}`,
      `Platform: ${navigator.userAgent}`,
      `Language: ${language}`,
    ].join("\n");
    void navigator.clipboard.writeText(diagnostics).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    });
  };

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <header className="mb-6 flex items-start gap-3">
        <AcornLogo size={32} />
        <div>
          <h1 className="text-[20px] font-semibold text-foreground">{t.aboutPanelTitle}</h1>
          <p className="mt-1 text-[13px] text-muted-foreground">{t.aboutPanelSubtitle}</p>
        </div>
      </header>

      <dl className="grid max-w-md grid-cols-[max-content_1fr] gap-x-6 gap-y-3 rounded-lg border-[0.5px] border-border bg-card px-4 py-3.5 text-[13px] shadow-[var(--shadow-card)]">
        <dt className="text-muted-foreground">{t.aboutVersionLabel}</dt>
        <dd className="font-mono tabular-nums text-foreground">
          {appName} {version || "…"}
        </dd>

        <dt className="text-muted-foreground">{t.aboutBuildLabel}</dt>
        <dd className="font-mono text-foreground/80">{buildMode}</dd>

        <dt className="text-muted-foreground">{t.aboutLicenseLabel}</dt>
        <dd className="font-mono text-foreground">{t.aboutLicenseValue}</dd>

        <dt className="text-muted-foreground">{t.aboutRepoLabel}</dt>
        <dd>
          <a
            href={REPO_URL}
            target="_blank"
            rel="noreferrer noopener"
            className="inline-flex items-center gap-1 text-acorn-orange hover:underline"
          >
            onyxcraft/acorn
            <ExternalLink className="w-3 h-3" />
          </a>
        </dd>
      </dl>

      <button
        type="button"
        onClick={copyDiagnostics}
        className="mt-6 inline-flex items-center gap-1.5 rounded-md border-[0.5px] border-border px-3 py-1 text-[13px] text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
      >
        {copied ? (
          <Check className="h-3.5 w-3.5 text-acorn-olive" />
        ) : (
          <Copy className="h-3.5 w-3.5" />
        )}
        {copied ? (zh ? "已复制" : "Copied") : zh ? "复制诊断信息" : "Copy diagnostics"}
      </button>
    </section>
  );
}
