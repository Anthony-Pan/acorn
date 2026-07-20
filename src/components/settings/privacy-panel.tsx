import { formatDistanceToNowStrict } from "date-fns";
import { Loader2, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { type ActivityEntry, activity } from "@/lib/activity";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

const KIND_LABELS_EN: Record<string, string> = {
  "chat:user-message": "Chat input",
};

const KIND_LABELS_ZH: Record<string, string> = {
  "chat:user-message": "聊天输入",
};

export function PrivacyPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);
  const labels = language.startsWith("zh") ? KIND_LABELS_ZH : KIND_LABELS_EN;

  const [entries, setEntries] = useState<ActivityEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const list = await activity.listRecent(100);
      setEntries(list);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleClear = async () => {
    setBusy(true);
    try {
      await activity.clear();
      setEntries([]);
      toast.success(t.privacyClearedToast);
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <header className="mb-5">
        <h1 className="text-[20px] font-semibold text-foreground">{t.privacyPanelTitle}</h1>
        <p className="mt-1 text-[13px] text-muted-foreground">{t.privacyPanelSubtitle}</p>
      </header>

      <div className="mb-4 flex max-w-3xl items-center gap-2">
        <Button variant="outline" size="sm" onClick={() => void refresh()} disabled={loading}>
          {loading ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <RefreshCw className="w-3.5 h-3.5" />
          )}
          {t.privacyRefreshButton}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={handleClear}
          disabled={busy || loading || entries.length === 0}
        >
          <Trash2 className="w-3.5 h-3.5" />
          {t.privacyClearButton}
        </Button>
        <span className="ml-auto text-[11px] text-muted-foreground">
          {t.privacyEntryCount(entries.length)}
        </span>
      </div>

      <div className="max-w-3xl">
        {entries.length === 0 && !loading ? (
          <div className="rounded-lg border-[0.5px] border-dashed border-border p-6 text-center text-[11px] text-muted-foreground">
            {t.privacyEmptyState}
          </div>
        ) : (
          <div className="divide-y-[0.5px] divide-border overflow-hidden rounded-lg border-[0.5px] border-border bg-card shadow-[var(--shadow-card)]">
            {entries.map((entry) => (
              <article key={entry.id} className="px-4 py-2.5">
                <div className="mb-1 flex items-center justify-between gap-3">
                  <span className="font-mono text-[10px] uppercase tracking-wider text-acorn-orange">
                    {labels[entry.kind] ?? entry.kind}
                  </span>
                  <span className="text-[11px] tabular-nums text-muted-foreground">
                    {formatDistanceToNowStrict(new Date(entry.createdAt), { addSuffix: true })}
                  </span>
                </div>
                <div className="line-clamp-2 break-words text-[13px] text-foreground/85">
                  {entry.content}
                </div>
              </article>
            ))}
          </div>
        )}
      </div>
    </section>
  );
}
