import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { motion } from "framer-motion";
import { Loader2, Mic, Send, Square } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { useRecorder } from "@/hooks/use-recorder";

type Phase = "idle" | "submitting";

export function QuickApp() {
  const [text, setText] = useState("");
  const [phase, setPhase] = useState<Phase>("idle");
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const { state: recState, start, stopAndTranscribe, cancel } = useRecorder();

  useEffect(() => {
    start().catch((err) => {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    });
  }, [start]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const dismiss = useCallback(async () => {
    cancel();
    setText("");
    setError(null);
    await getCurrentWindow().hide();
  }, [cancel]);

  const submit = useCallback(
    async (value: string) => {
      const trimmed = value.trim();
      if (!trimmed || phase === "submitting") return;
      setPhase("submitting");
      try {
        await emit("quick:submit", { text: trimmed });
        setText("");
        setError(null);
        await getCurrentWindow().hide();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setPhase("idle");
      }
    },
    [phase],
  );

  const handleAction = useCallback(async () => {
    if (recState === "recording") {
      try {
        const transcript = await stopAndTranscribe();
        if (transcript.trim()) {
          await submit(transcript);
        } else {
          await dismiss();
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
      return;
    }
    await submit(text);
  }, [recState, stopAndTranscribe, submit, dismiss, text]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        void dismiss();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [dismiss]);

  const hint =
    recState === "recording"
      ? "Listening..."
      : recState === "transcribing"
        ? "Transcribing..."
        : "Speak or type. Press Esc to dismiss.";

  const buttonIcon = (() => {
    if (phase === "submitting" || recState === "transcribing") {
      return <Loader2 className="w-4 h-4 animate-spin" />;
    }
    if (recState === "recording") {
      return <Square className="w-3 h-3 fill-current" />;
    }
    if (text.trim()) {
      return <Send className="w-4 h-4" />;
    }
    return <Mic className="w-4 h-4" />;
  })();

  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.18, ease: "easeOut" }}
      className="flex items-center justify-center w-screen h-screen p-3"
    >
      <div
        className="w-full max-w-2xl rounded-3xl px-5 py-4 shadow-2xl border border-acorn-brown/15"
        style={{
          background: "rgba(250, 244, 236, 0.82)",
          backdropFilter: "blur(40px) saturate(180%)",
          WebkitBackdropFilter: "blur(40px) saturate(180%)",
        }}
      >
        <div className="flex items-center gap-2.5 mb-2">
          <AcornLogo size={22} />
          <div className="text-[11px] text-muted-foreground">{hint}</div>
        </div>

        <div className="flex items-center gap-3">
          <input
            ref={inputRef}
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                cancel();
                void submit(text);
              }
            }}
            placeholder="What's on your mind right now?"
            className="flex-1 bg-transparent text-[15px] text-foreground placeholder:text-muted-foreground/60 focus:outline-none"
            disabled={phase === "submitting"}
          />

          <button
            type="button"
            onClick={() => void handleAction()}
            disabled={phase === "submitting"}
            className="flex items-center justify-center w-9 h-9 rounded-full bg-acorn-brown text-acorn-paper hover:bg-acorn-brown-deep transition-colors disabled:opacity-50"
            aria-label={recState === "recording" ? "Stop recording" : "Submit"}
          >
            {buttonIcon}
          </button>
        </div>

        {error ? <div className="mt-2 text-[11px] text-acorn-red">{error}</div> : null}
      </div>
    </motion.div>
  );
}
