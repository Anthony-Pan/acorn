import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { Check, Loader2, Wrench } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { cn } from "@/lib/utils";
import { overlay } from "@/lib/window";
import type { OverlayPhase } from "@/types/overlay";

const NOTCH_LABEL = "notch";
const DONE_VISIBLE_MS = 2600;

const IDLE: OverlayPhase = { phase: "idle", tool: null, toolCount: 0, providerId: null };

/**
 * The floating status capsule that lives in its own top-center overlay window.
 * It mirrors the main window's chat activity (received over `overlay:phase`)
 * and renders nothing when idle, so the DOM is static and the transparent
 * webview costs nothing at rest.
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

      // Draw a brief completion check when active work settles back to idle.
      if (next.phase === "idle" && was !== "idle") {
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

  const visible = state.phase !== "idle" || done;

  const handleEnter = () => {
    setExpanded(true);
    void overlay.setInteractive(NOTCH_LABEL, true);
  };
  const handleLeave = () => {
    setExpanded(false);
    void overlay.setInteractive(NOTCH_LABEL, false);
  };

  const label =
    state.phase === "thinking"
      ? "Thinking"
      : state.phase === "tool"
        ? `Running ${state.tool ?? "tool"}`
        : state.phase === "responding"
          ? "Replying"
          : done
            ? "Done"
            : "";

  return (
    <div className="fixed inset-x-0 top-0 flex justify-center pt-1.5">
      <AnimatePresence>
        {visible ? (
          <motion.div
            key="capsule"
            initial={{ opacity: 0, y: -10, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -10, scale: 0.9 }}
            transition={{ type: "spring", stiffness: 380, damping: 30 }}
            onPointerEnter={handleEnter}
            onPointerLeave={handleLeave}
            style={{ pointerEvents: "auto" }}
            className={cn(
              "px-3 py-1 rounded-full border-[0.5px] bg-card/95 backdrop-blur",
              "border-acorn-orange/40 shadow-sm flex items-center gap-2 text-[11px] select-none",
            )}
          >
            {done ? (
              <Check className="w-3 h-3 text-acorn-olive" />
            ) : state.phase === "tool" ? (
              <Wrench className="w-3 h-3 text-acorn-orange" />
            ) : (
              <Loader2 className="w-3 h-3 animate-spin text-acorn-orange" />
            )}
            <span className="font-medium text-foreground">{label}</span>
            {expanded && state.providerId ? (
              <>
                <span className="text-muted-foreground/60">·</span>
                <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-wider">
                  {state.providerId}
                </span>
              </>
            ) : null}
            {expanded && state.toolCount > 1 ? (
              <>
                <span className="text-muted-foreground/60">·</span>
                <span className="text-muted-foreground">+{state.toolCount - 1}</span>
              </>
            ) : null}
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
