import { Loader2, Save, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { settings } from "@/lib/db";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

export const SHARED_MEMORY_KEY = "shared_memory";

export function MemoryPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const [value, setValue] = useState("");
  const [savedValue, setSavedValue] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const stored = await settings.get(SHARED_MEMORY_KEY);
      const text = stored ?? "";
      setValue(text);
      setSavedValue(text);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const dirty = value !== savedValue;

  const handleSave = async () => {
    setSaving(true);
    try {
      const trimmed = value.trim();
      if (trimmed.length === 0) {
        await settings.remove(SHARED_MEMORY_KEY);
      } else {
        await settings.set(SHARED_MEMORY_KEY, trimmed);
      }
      setSavedValue(trimmed);
      setValue(trimmed);
      toast.success(t.memorySavedToast);
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    setSaving(true);
    try {
      await settings.remove(SHARED_MEMORY_KEY);
      setValue("");
      setSavedValue("");
      toast.success(t.memoryClearedToast);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <header className="mb-5">
        <h1 className="text-[20px] font-semibold text-foreground">{t.memoryPanelTitle}</h1>
        <p className="mt-1 text-[13px] text-muted-foreground">{t.memoryPanelSubtitle}</p>
      </header>

      <textarea
        value={value}
        onChange={(e) => setValue(e.target.value)}
        disabled={loading}
        placeholder={t.memoryPlaceholder}
        className="h-64 w-full max-w-2xl resize-y rounded-lg border-[0.5px] border-border bg-card px-3.5 py-3 font-mono text-[13px] shadow-[var(--shadow-card)] focus:border-ring/60 focus:outline-none"
      />

      <div className="mt-3 flex max-w-2xl items-center gap-2">
        <Button onClick={handleSave} disabled={!dirty || saving || loading} size="sm">
          {saving ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <Save className="w-3.5 h-3.5" />
          )}
          {t.memorySaveButton}
        </Button>
        <Button
          variant="outline"
          onClick={handleClear}
          disabled={saving || loading || savedValue.length === 0}
          size="sm"
        >
          <Trash2 className="w-3.5 h-3.5" />
          {t.memoryClearButton}
        </Button>
        <span className="ml-auto text-[11px] text-muted-foreground">{t.memoryHint}</span>
      </div>
    </section>
  );
}
