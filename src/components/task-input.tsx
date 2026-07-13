import {
  AlertCircle,
  CalendarDays,
  Clipboard,
  FileText,
  Loader2,
  MessageCircle,
  Settings as SettingsIcon,
  Zap,
} from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { AcornLogo } from "@/components/acorn-logo";
import { Button } from "@/components/ui/button";
import { VoiceButton } from "@/components/voice-button";
import { formatToday } from "@/lib/format";
import { strings } from "@/lib/i18n";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";

interface TaskInputProps {
  onOpenSettings: () => void;
  onOpenChat: () => void;
  onOpenCalendar: () => void;
  onOpenCanvas: () => void;
}

function pickHint(error: string | null, t: ReturnType<typeof strings>): string {
  if (!error) return t.stashFailedHint;
  const lower = error.toLowerCase();
  const modelMatch = lower.match(/model ['"]([^'"]+)['"] not found/);
  if (modelMatch?.[1]) return t.hintModelNotFound(modelMatch[1]);
  if (lower.includes("ollama not running") || lower.includes("connection refused")) {
    return t.hintOllamaDown;
  }
  if (
    lower.includes("invalid api key") ||
    lower.includes("401") ||
    lower.includes("unauthorized")
  ) {
    return t.hintInvalidKey;
  }
  return t.stashFailedHint;
}

export function TaskInput({
  onOpenSettings,
  onOpenChat,
  onOpenCalendar,
  onOpenCanvas,
}: TaskInputProps) {
  const [value, setValue] = useState(() => {
    const pending = typeof window !== "undefined" ? sessionStorage.getItem("acorn:prefill") : null;
    if (pending) {
      sessionStorage.removeItem("acorn:prefill");
      return pending;
    }
    return "";
  });
  const [voiceError, setVoiceError] = useState<string | null>(null);

  const isStashing = useSessionStore((s) => s.isStashing);
  const stashError = useSessionStore((s) => s.error);
  const progress = useSessionStore((s) => s.progress);
  const stash = useSessionStore((s) => s.stash);
  const activeProviderId = useSettingsStore((s) => s.activeProviderId);
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const trimmed = value.trim();
  const canStash = trimmed.length > 0 && !!activeProviderId && !isStashing;

  useEffect(() => {
    if (stashError && !isStashing) {
      toast.error(t.stashFailedTitle, {
        id: "stash-error",
        description: stashError,
      });
    }
  }, [stashError, isStashing, t.stashFailedTitle]);

  const errorHint = pickHint(stashError, t);

  const handleStash = async () => {
    if (!canStash || !activeProviderId) return;
    await stash(trimmed, activeProviderId, language);
    if (!useSessionStore.getState().error) {
      setValue("");
      toast.success(t.stashSuccess);
    }
  };

  const handleVoiceTranscript = (text: string) => {
    setVoiceError(null);
    setValue((v) => (v ? `${v}\n${text}` : text));
  };

  const handlePaste = async () => {
    const clip = await navigator.clipboard.readText().catch(() => null);
    if (clip) setValue((v) => (v ? `${v}\n${clip}` : clip));
  };

  return (
    <div className="min-h-screen bg-background">
      <header
        data-tauri-drag-region
        className="flex h-[52px] items-center justify-between pl-[76px] pr-4 select-none"
      >
        <div className="pointer-events-none flex items-center gap-2.5">
          <AcornLogo size={24} />
          <div className="flex items-baseline gap-2">
            <div className="text-[13px] font-semibold text-foreground leading-none">Acorn</div>
            <div className="text-[11px] text-muted-foreground leading-none">{formatToday()}</div>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={onOpenChat}>
            <MessageCircle />
            {t.chat}
          </Button>
          <Button variant="outline" size="icon-sm" onClick={onOpenCanvas} aria-label="Canvas">
            <FileText className="w-3.5 h-3.5" />
          </Button>
          <Button variant="outline" size="icon-sm" onClick={onOpenCalendar} aria-label="Calendar">
            <CalendarDays />
          </Button>
          <Button variant="outline" size="icon-sm" onClick={onOpenSettings} aria-label="Settings">
            <SettingsIcon />
          </Button>
        </div>
      </header>

      <div className="max-w-2xl mx-auto px-7 pb-7 pt-6">
        <h1 className="text-[20px] font-semibold tracking-tight text-foreground mb-1.5 leading-tight">
          {t.headingPrimary}
        </h1>
        <p className="text-[13px] text-muted-foreground mb-5">{t.headingSecondary}</p>

        <textarea
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
              e.preventDefault();
              if (canStash) void handleStash();
            }
          }}
          placeholder={t.inputPlaceholder}
          disabled={isStashing}
          className="w-full min-h-[180px] bg-card border-[0.5px] border-border rounded-lg shadow-[var(--shadow-card)] px-4.5 py-4 text-[13px] leading-[1.7] resize-none focus:outline-none focus:ring-2 focus:ring-ring/30 placeholder:text-[var(--acorn-orange-muted)]/60 disabled:opacity-60"
        />

        <div className="flex items-center justify-between mt-3.5">
          <div className="flex gap-1.5">
            <VoiceButton
              onTranscript={handleVoiceTranscript}
              onError={(err) => setVoiceError(err.message)}
            />
            <Button variant="outline" size="sm" onClick={handlePaste}>
              <Clipboard />
              {t.paste}
            </Button>
          </div>
          <Button
            onClick={handleStash}
            disabled={!canStash}
            size="lg"
            className="px-5 font-medium text-[13px]"
          >
            {isStashing ? (
              <span className="inline-flex items-center gap-2 tabular-nums">
                <Loader2 className="w-4 h-4 animate-spin" />
                {t.stashing}
                {progress > 0 ? ` · ${progress.toLocaleString()}` : ""}
              </span>
            ) : (
              t.stashIt
            )}
          </Button>
        </div>

        {voiceError ? (
          <div className="mt-3 text-[13px] text-acorn-red flex items-start gap-1.5">
            <AlertCircle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
            <span>{voiceError}</span>
          </div>
        ) : null}

        {stashError && !isStashing ? (
          <div className="mt-3 p-3 rounded-lg bg-acorn-red/8 border-[0.5px] border-acorn-red/25 text-[13px] text-acorn-red flex items-start gap-2">
            <AlertCircle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
            <div className="flex-1">
              <div className="font-medium mb-0.5">{t.stashFailedTitle}</div>
              <div className="opacity-90 leading-relaxed break-words">{stashError}</div>
              <div className="mt-1.5 text-[11px] opacity-70">{errorHint}</div>
            </div>
          </div>
        ) : null}

        {!activeProviderId ? (
          <div className="mt-3 text-[13px] text-muted-foreground">
            <button type="button" className="underline" onClick={onOpenSettings}>
              {t.configureProvider}
            </button>{" "}
            {t.configureProviderTail}
          </div>
        ) : null}

        <footer className="flex items-center gap-2 pt-4 mt-5 border-t-[0.5px] border-border">
          <Zap className="w-3.5 h-3.5 text-acorn-orange" />
          <div className="text-[11px] text-muted-foreground">
            {t.summonHint}{" "}
            <kbd className="px-1.5 py-0.5 bg-muted rounded font-mono text-[11px]">⌘ ⇧ A</kbd>{" "}
            {t.summonHintTail}
          </div>
        </footer>
      </div>
    </div>
  );
}
