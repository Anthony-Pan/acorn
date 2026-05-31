import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Loader2, Mic, Square } from "lucide-react";
import { useEffect } from "react";

import { VoiceHalo } from "@/components/voice-halo";
import { useRecorder } from "@/hooks/use-recorder";
import { cn } from "@/lib/utils";

interface VoiceButtonProps {
  onTranscript: (text: string) => void;
  onError?: (err: Error) => void;
}

export function VoiceButton({ onTranscript, onError }: VoiceButtonProps) {
  const { state, start, stopAndTranscribe, cancel } = useRecorder();

  // Mirror voice capture to the dynamic island so the mic state is visible even
  // when this window is behind another app.
  useEffect(() => {
    void emit("overlay:voice", { state });
  }, [state]);

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

  useEffect(() => {
    const unlisten = listen("shortcut:push-to-talk", async () => {
      try {
        const win = getCurrentWindow();
        await win.show();
        await win.setFocus();
        if (state === "idle") {
          await start();
        } else if (state === "recording") {
          const text = await stopAndTranscribe();
          if (text.trim()) onTranscript(text.trim());
        }
      } catch (err) {
        cancel();
        onError?.(err instanceof Error ? err : new Error(String(err)));
      }
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [state, start, stopAndTranscribe, cancel, onTranscript, onError]);

  const label =
    state === "recording" ? "Stop" : state === "transcribing" ? "Transcribing" : "Voice";

  return (
    <>
      <VoiceHalo visible={state === "recording"} />
      <button
        type="button"
        onClick={handleClick}
        disabled={state === "transcribing"}
        className={cn(
          "inline-flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-md transition-colors",
          "border-[0.5px] border-border",
          state === "recording" && "bg-acorn-red text-acorn-paper border-transparent animate-pulse",
          state === "idle" && "text-muted-foreground hover:bg-muted",
          state === "transcribing" && "text-muted-foreground opacity-70",
        )}
      >
        {state === "recording" && <Square className="w-3 h-3 fill-current" />}
        {state === "idle" && <Mic className="w-3.5 h-3.5" />}
        {state === "transcribing" && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
        {label}
      </button>
    </>
  );
}
