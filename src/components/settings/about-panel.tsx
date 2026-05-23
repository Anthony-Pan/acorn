import { getName, getVersion } from "@tauri-apps/api/app";
import { ExternalLink } from "lucide-react";
import { useEffect, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

const REPO_URL = "https://github.com/onyxcraft/acorn";

export function AboutPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const [appName, setAppName] = useState<string>("Acorn");
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    void getName().then(setAppName);
    void getVersion().then(setVersion);
  }, []);

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-6 flex items-start gap-3">
        <AcornLogo size={32} />
        <div>
          <h1 className="text-lg font-medium text-foreground">{t.aboutPanelTitle}</h1>
          <p className="text-xs text-muted-foreground mt-1">{t.aboutPanelSubtitle}</p>
        </div>
      </header>

      <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-3 text-sm max-w-md">
        <dt className="text-muted-foreground">{t.aboutVersionLabel}</dt>
        <dd className="font-mono text-foreground">
          {appName} {version || "…"}
        </dd>

        <dt className="text-muted-foreground">{t.aboutBuildLabel}</dt>
        <dd className="font-mono text-foreground/80">
          {import.meta.env.MODE === "production" ? "release" : "dev"}
        </dd>

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
    </section>
  );
}
