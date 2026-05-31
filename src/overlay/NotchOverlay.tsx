import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { AlertTriangle, Check, Loader2, Wrench } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { cn } from "@/lib/utils";
import { overlay } from "@/lib/window";
import type { OverlayPhase } from "@/types/overlay";
import { CornieSprite } from "./cornie-sprite";

const NOTCH_LABEL = "notch";
const DONE_VISIBLE_MS = 2600;

const IDLE: OverlayPhase = {
  phase: "idle",
  tool: null,
  toolCount: 0,
  providerId: null,
  model: null,
  lastReply: null,
  error: null,
};

/**
 * Acorn's dynamic island: a floating top-center capsule in its own overlay
 * window. The left "ear" carries a mini Cornie that animates with chat activity;
 * the body shows the current state (thinking / running a tool / replying / done /
 * error); hovering expands it like a water drop to reveal the provider, model,
 * and a preview of the latest reply. It mirrors the main window's chat store over
 * the `overlay:phase` event and renders nothing when idle, so a resting island
 * costs nothing to repaint.
 */
export function NotchOverlay() {
  const [state, setState] = useState<OverlayPhase>(IDLE);
  const [done, setDone] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const prevPhase = useRef<OverlayPhase["phase"]>("idle");
  const doneTimer = useRef<number | null>(null);

  useEffect(() => {
    const unlisten = listen<OverlayPhase>("overlay:phase", (event) => {
      const next = event.payload;
      const was = prevPhase.current;
      prevPhase.current = next.phase;

      if (next.phase === "idle" && was !== "idle" && !next.error) {
        setDone(true);
        if (doneTimer.current) window.clearTimeout(doneTimer.current);
        doneTimer.current = window.setTimeout(() => setDone(false), DONE_VISIBLE_MS);
      } else if (next.phase !== "idle") {
        setDone(false);
      }
      setState(next);
    });
    return () => {
      void unlisten.then((fn) => fn());
      if (doneTimer.current) window.clearTimeout(doneTimer.current);
    };
  }, []);

  const hasError = Boolean(state.error) && state.phase === "idle";
  const active = state.phase !== "idle";
  const visible = active || done || hasError;

  // Drive interactivity from content visibility: a fully click-through window
  // gets no pointer events, so the capsule captures clicks only while shown.
  useEffect(() => {
    void overlay.setInteractive(NOTCH_LABEL, visible);
  }, [visible]);

  const spriteAnimation = active ? "think" : "bounce";

  const label = hasError
    ? "Something went wrong"
    : state.phase === "thinking"
      ? "Thinking"
      : state.phase === "tool"
        ? `Running ${state.tool ?? "tool"}`
        : state.phase === "responding"
          ? "Replying"
          : done
            ? "Done"
            : "";

  const openChat = () => {
    void invoke("show_main").catch(() => {});
    void emit("navigate", "chat");
  };

  return (
    <div className="fixed inset-x-0 top-0 flex justify-center pt-1.5">
      <AnimatePresence>
        {visible ? (
          <motion.button
            type="button"
            key="capsule"
            initial={{ opacity: 0, y: -10, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -10, scale: 0.9 }}
            transition={{ type: "spring", stiffness: 380, damping: 30 }}
            onPointerEnter={() => setExpanded(true)}
            onPointerLeave={() => setExpanded(false)}
            onClick={hasError ? openChat : undefined}
            style={{ pointerEvents: "auto" }}
            className={cn(
              "flex max-w-[460px] items-center gap-2 rounded-full border-[0.5px] px-2.5 py-1",
              "bg-card/95 text-[11px] shadow-sm backdrop-blur select-none",
              hasError ? "border-acorn-red/50 cursor-pointer" : "border-acorn-orange/40",
            )}
          >
            <span className="flex h-4 w-4 shrink-0 items-center justify-center">
              <CornieSprite size={14} animation={spriteAnimation} />
            </span>

            {hasError ? (
              <AlertTriangle className="h-3 w-3 shrink-0 text-acorn-red" />
            ) : done ? (
              <Check className="h-3 w-3 shrink-0 text-acorn-olive" />
            ) : state.phase === "tool" ? (
              <Wrench className="h-3 w-3 shrink-0 text-acorn-orange" />
            ) : (
              <Loader2 className="h-3 w-3 shrink-0 animate-spin text-acorn-orange" />
            )}

            <span className="truncate font-medium text-foreground">{label}</span>

            {!hasError && state.toolCount > 1 ? (
              <span className="shrink-0 text-muted-foreground">+{state.toolCount - 1}</span>
            ) : null}

            <AnimatePresence>
              {expanded && (state.providerId || state.lastReply || hasError) ? (
                <motion.span
                  key="expand"
                  initial={{ opacity: 0, width: 0 }}
                  animate={{ opacity: 1, width: "auto" }}
                  exit={{ opacity: 0, width: 0 }}
                  transition={{ type: "spring", stiffness: 300, damping: 30 }}
                  className="flex items-center gap-1.5 overflow-hidden"
                >
                  {state.providerId ? (
                    <>
                      <span className="text-muted-foreground/50">·</span>
                      <span className="shrink-0 font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
                        {state.providerId}
                        {state.model ? ` · ${state.model}` : ""}
                      </span>
                    </>
                  ) : null}
                  {hasError ? (
                    <span className="max-w-[220px] truncate text-acorn-red/90">{state.error}</span>
                  ) : state.lastReply ? (
                    <>
                      <span className="text-muted-foreground/50">·</span>
                      <span className="max-w-[240px] truncate text-muted-foreground">
                        {state.lastReply}
                      </span>
                    </>
                  ) : null}
                </motion.span>
              ) : null}
            </AnimatePresence>
          </motion.button>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
