import { Clipboard, MessageCircle, Settings as SettingsIcon, Zap } from "lucide-react";
import { useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { Button } from "@/components/ui/button";
import { VoiceButton } from "@/components/voice-button";
import { formatToday } from "@/lib/format";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";

interface TaskInputProps {
  onOpenSettings: () => void;
  onOpenChat: () => void;
}

export function TaskInput({ onOpenSettings, onOpenChat }: TaskInputProps) {
  const [value, setValue] = useState("");
  const [voiceError, setVoiceError] = useState<string | null>(null);

  const isStashing = useSessionStore((s) => s.isStashing);
  const stash = useSessionStore((s) => s.stash);
  const activeProviderId = useSettingsStore((s) => s.activeProviderId);
  const language = useSettingsStore((s) => s.language);

  const trimmed = value.trim();
  const canStash = trimmed.length > 0 && !!activeProviderId && !isStashing;

  const handleStash = async () => {
    if (!canStash || !activeProviderId) return;
    try {
      await stash(trimmed, activeProviderId, language);
      setValue("");
    } catch (err) {
      console.error("stash failed", err);
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
              Chat
            </Button>
            <Button variant="outline" size="icon-sm" onClick={onOpenSettings} aria-label="Settings">
              <SettingsIcon />
            </Button>
          </div>
        </header>

        <h1 className="text-[22px] font-medium text-foreground mb-1.5 leading-tight">
          What's on your mind today?
        </h1>
        <p className="text-[13px] text-muted-foreground mb-5">
          今天脑子里都有什么?一股脑倒出来,Acorn 帮你整理。
        </p>

        <textarea
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="Like: I need to call Mr. Zhang about the proposal, finish the weekly report, review product analytics from yesterday, prepare for my 7pm interview, and somewhere in there grocery shop..."
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
              Paste
            </Button>
          </div>
          <Button onClick={handleStash} disabled={!canStash} size="lg" className="px-5 font-medium">
            {isStashing ? "Stashing..." : "Stash it 🌰"}
          </Button>
        </div>

        {voiceError ? <div className="mt-3 text-xs text-acorn-red">{voiceError}</div> : null}

        {!activeProviderId ? (
          <div className="mt-3 text-xs text-muted-foreground">
            <button type="button" className="underline" onClick={onOpenSettings}>
              Configure a provider
            </button>{" "}
            in Settings before you can stash.
          </div>
        ) : null}

        <footer className="flex items-center gap-2 pt-4 mt-5 border-t-[0.5px] border-acorn-brown/10">
          <Zap className="w-3.5 h-3.5 text-acorn-orange" />
          <div className="text-xs text-muted-foreground">
            Press <kbd className="px-1.5 py-0.5 bg-muted rounded font-mono text-[11px]">⌘ ⇧ A</kbd>{" "}
            anywhere to summon Acorn
          </div>
        </footer>
      </div>
    </div>
  );
}
