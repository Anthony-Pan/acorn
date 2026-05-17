import { Loader2, Mic, Square } from "lucide-react";

import { useRecorder } from "@/hooks/use-recorder";
import { cn } from "@/lib/utils";

interface VoiceButtonProps {
  onTranscript: (text: string) => void;
  onError?: (err: Error) => void;
}

export function VoiceButton({ onTranscript, onError }: VoiceButtonProps) {
  const { state, start, stopAndTranscribe, cancel } = useRecorder();

  const handleClick = async () => {
    try {
      if (state === "idle") {
        await start();
        return;
      }
      if (state === "recording") {
        const text = await stopAndTranscribe();
        if (text.trim()) onTranscript(text.trim());
      }
    } catch (err) {
      cancel();
      onError?.(err instanceof Error ? err : new Error(String(err)));
    }
  };

  const label =
    state === "recording" ? "Stop" : state === "transcribing" ? "Transcribing" : "Voice";

  return (
    <button
      type="button"
      onClick={handleClick}
      disabled={state === "transcribing"}
      className={cn(
        "inline-flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-md transition-colors",
        "border-[0.5px] border-border",
        state === "recording" && "bg-acorn-red text-acorn-paper border-transparent",
        state === "idle" && "text-muted-foreground hover:bg-muted",
        state === "transcribing" && "text-muted-foreground opacity-70",
      )}
    >
      {state === "recording" && <Square className="w-3 h-3 fill-current" />}
      {state === "idle" && <Mic className="w-3.5 h-3.5" />}
      {state === "transcribing" && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
      {label}
    </button>
  );
}
