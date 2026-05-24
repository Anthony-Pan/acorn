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
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-5">
        <h1 className="text-lg font-medium text-foreground">{t.privacyPanelTitle}</h1>
        <p className="text-xs text-muted-foreground mt-1">{t.privacyPanelSubtitle}</p>
      </header>

      <div className="flex items-center gap-2 mb-4 max-w-3xl">
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

      <div className="space-y-1.5 max-w-3xl">
        {entries.length === 0 && !loading ? (
          <div className="border-[0.5px] border-dashed border-border rounded-md p-6 text-center text-xs text-muted-foreground">
            {t.privacyEmptyState}
          </div>
        ) : (
          entries.map((entry) => (
            <article
              key={entry.id}
              className="border-[0.5px] border-border rounded-md px-3 py-2 bg-card"
            >
              <div className="flex items-center justify-between gap-3 mb-1">
                <span className="text-[10px] font-mono uppercase tracking-wider text-acorn-orange">
                  {labels[entry.kind] ?? entry.kind}
                </span>
                <span className="text-[10px] text-muted-foreground">
                  {formatDistanceToNowStrict(new Date(entry.createdAt), { addSuffix: true })}
                </span>
              </div>
              <div className="text-xs text-foreground/85 break-words line-clamp-2">
                {entry.content}
              </div>
            </article>
          ))
        )}
      </div>
    </section>
  );
}
