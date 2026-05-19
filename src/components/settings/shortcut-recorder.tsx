import { invoke } from "@tauri-apps/api/core";
import { Loader2, RotateCcw } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { strings } from "@/lib/i18n";

interface ShortcutRecorderProps {
  language: string;
}

type Phase = "idle" | "recording" | "saving";

const MODIFIER_KEYS = new Set(["Meta", "Control", "Shift", "Alt", "OS", "AltGraph"]);

export function ShortcutRecorder({ language }: ShortcutRecorderProps) {
  const t = strings(language);
  const [shortcut, setShortcut] = useState<string>("CmdOrCtrl+Shift+KeyA");
  const [phase, setPhase] = useState<Phase>("idle");
  const [draft, setDraft] = useState<string | null>(null);
  const recorderRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    invoke<string>("get_summon_shortcut")
      .then(setShortcut)
      .catch(() => setShortcut("CmdOrCtrl+Shift+KeyA"));
  }, []);

  useEffect(() => {
    if (phase !== "recording") return;
    const handler = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        setDraft(null);
        setPhase("idle");
        return;
      }
      if (MODIFIER_KEYS.has(event.key)) return;
      const parts: string[] = [];
      if (event.metaKey || event.ctrlKey) parts.push("CmdOrCtrl");
      if (event.shiftKey) parts.push("Shift");
      if (event.altKey) parts.push("Alt");
      if (parts.length === 0) return;
      parts.push(event.code);
      setDraft(parts.join("+"));
    };
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [phase]);

  const startRecording = () => {
    setDraft(null);
    setPhase("recording");
    setTimeout(() => recorderRef.current?.focus(), 0);
  };

  const cancelRecording = () => {
    setDraft(null);
    setPhase("idle");
  };

  const saveShortcut = async () => {
    if (!draft) return;
    setPhase("saving");
    try {
      await invoke<void>("set_summon_shortcut", { shortcut: draft });
      setShortcut(draft);
      setDraft(null);
      setPhase("idle");
      toast.success(`Shortcut saved: ${formatShortcut(draft)}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error("Could not save shortcut", { description: message });
      setPhase("recording");
    }
  };

  const resetShortcut = async () => {
    setPhase("saving");
    try {
      const next = await invoke<string>("reset_summon_shortcut");
      setShortcut(next);
      setDraft(null);
      setPhase("idle");
      toast.success(`Reset to ${formatShortcut(next)}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error("Could not reset", { description: message });
      setPhase("idle");
    }
  };

  return (
    <div className="flex items-center gap-3">
      <div>
        <div className="text-sm">{t.shortcutLabel}</div>
        <div className="text-[11px] text-muted-foreground">{t.shortcutCaption}</div>
      </div>
      <button
        ref={recorderRef}
        type="button"
        onClick={phase === "idle" ? startRecording : undefined}
        disabled={phase === "saving"}
        className="bg-card border-[0.5px] border-border rounded-md px-3 py-1.5 text-sm font-mono min-w-[140px] text-left focus:outline-none focus:ring-2 focus:ring-ring/30"
      >
        {phase === "recording"
          ? draft
            ? formatShortcut(draft)
            : "Press a combination..."
          : formatShortcut(shortcut)}
      </button>
      {phase === "recording" ? (
        <>
          <Button onClick={saveShortcut} disabled={!draft} size="sm">
            {t.saveShortcut}
          </Button>
          <Button onClick={cancelRecording} variant="outline" size="sm">
            {t.cancelRecording}
          </Button>
        </>
      ) : (
        <Button
          onClick={resetShortcut}
          variant="outline"
          size="icon-sm"
          disabled={phase === "saving"}
          aria-label={t.resetShortcut}
        >
          {phase === "saving" ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <RotateCcw className="w-3.5 h-3.5" />
          )}
        </Button>
      )}
    </div>
  );
}

function formatShortcut(raw: string): string {
  return raw
    .split("+")
    .map((part) => {
      if (part === "CmdOrCtrl") return "⌘";
      if (part === "Meta" || part === "Super") return "⌘";
      if (part === "Control") return "⌃";
      if (part === "Ctrl") return "⌃";
      if (part === "Shift") return "⇧";
      if (part === "Alt") return "⌥";
      if (part === "Option") return "⌥";
      if (part.startsWith("Key")) return part.slice(3);
      if (part.startsWith("Digit")) return part.slice(5);
      if (part === "Space") return "Space";
      return part;
    })
    .join("");
}
