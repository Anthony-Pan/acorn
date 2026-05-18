import {
  AlertCircle,
  CalendarDays,
  Clipboard,
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
}

export function TaskInput({ onOpenSettings, onOpenChat, onOpenCalendar }: TaskInputProps) {
  const [value, setValue] = useState("");
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
      toast.error(t.stashFailedTitle, { description: stashError });
    }
  }, [stashError, isStashing, t.stashFailedTitle]);

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
    <div className="min-h-screen bg-background p-7">
      <div className="max-w-2xl mx-auto">
        <header className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-2.5">
            <AcornLogo size={32} />
            <div>
              <div className="text-base font-medium text-foreground leading-tight">Acorn</div>
              <div className="text-[11px] text-muted-foreground">{formatToday()}</div>
            </div>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={onOpenChat}>
              <MessageCircle />
              {t.chat}
            </Button>
            <Button variant="outline" size="icon-sm" onClick={onOpenCalendar} aria-label="Calendar">
              <CalendarDays />
            </Button>
            <Button variant="outline" size="icon-sm" onClick={onOpenSettings} aria-label="Settings">
              <SettingsIcon />
            </Button>
          </div>
        </header>

        <h1 className="text-[22px] font-medium text-foreground mb-1.5 leading-tight">
          {t.headingPrimary}
        </h1>
        <p className="text-[13px] text-muted-foreground mb-5">{t.headingSecondary}</p>

        <textarea
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder={t.inputPlaceholder}
          disabled={isStashing}
          className="w-full min-h-[180px] bg-card border-[0.5px] border-border rounded-2xl px-4.5 py-4 text-[14px] leading-[1.7] resize-none focus:outline-none focus:ring-2 focus:ring-ring/30 placeholder:text-[var(--acorn-orange-muted)]/60 disabled:opacity-60"
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
          <Button onClick={handleStash} disabled={!canStash} size="lg" className="px-5 font-medium">
            {isStashing ? (
              <span className="inline-flex items-center gap-2">
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
          <div className="mt-3 text-xs text-acorn-red flex items-start gap-1.5">
            <AlertCircle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
            <span>{voiceError}</span>
          </div>
        ) : null}

        {stashError && !isStashing ? (
          <div className="mt-3 p-3 rounded-lg bg-acorn-red/8 border border-acorn-red/20 text-xs text-acorn-red flex items-start gap-2">
            <AlertCircle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
            <div className="flex-1">
              <div className="font-medium mb-0.5">{t.stashFailedTitle}</div>
              <div className="opacity-90 leading-relaxed break-words">{stashError}</div>
              <div className="mt-1.5 opacity-70">{t.stashFailedHint}</div>
            </div>
          </div>
        ) : null}

        {!activeProviderId ? (
          <div className="mt-3 text-xs text-muted-foreground">
            <button type="button" className="underline" onClick={onOpenSettings}>
              {t.configureProvider}
            </button>{" "}
            {t.configureProviderTail}
          </div>
        ) : null}

        <footer className="flex items-center gap-2 pt-4 mt-5 border-t-[0.5px] border-acorn-brown/10">
          <Zap className="w-3.5 h-3.5 text-acorn-orange" />
          <div className="text-xs text-muted-foreground">
            {t.summonHint}{" "}
            <kbd className="px-1.5 py-0.5 bg-muted rounded font-mono text-[11px]">⌘ ⇧ A</kbd>{" "}
            {t.summonHintTail}
          </div>
        </footer>
      </div>
    </div>
  );
}
