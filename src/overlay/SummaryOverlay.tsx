import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { Check, Copy, MessageSquare } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { overlay } from "@/lib/window";
import type { SummaryPayload } from "@/types/overlay";

const SUMMARY_LABEL = "summary";
const VISIBLE_MS = 8000;
const COPIED_VISIBLE_MS = 1500;

/**
 * A transient reply summary shown top-center when a reply completes while the
 * chat view is not in focus. Auto-dismisses after 8s (hover pauses), offers
 * Copy and Open-full. The window is hidden (not destroyed) on dismiss, so this
 * surface and its listener persist for the next reply.
 */
export function SummaryOverlay() {
  const [data, setData] = useState<SummaryPayload | null>(null);
  const [copied, setCopied] = useState(false);
  const dismissTimer = useRef<number | null>(null);
  const copyTimer = useRef<number | null>(null);

  const clearDismiss = useCallback(() => {
    if (dismissTimer.current) {
      window.clearTimeout(dismissTimer.current);
      dismissTimer.current = null;
    }
  }, []);

  const dismiss = useCallback(() => {
    clearDismiss();
    setData(null);
    void overlay.setInteractive(SUMMARY_LABEL, false);
    void overlay.hideSummary();
  }, [clearDismiss]);

  const scheduleDismiss = useCallback(() => {
    clearDismiss();
    dismissTimer.current = window.setTimeout(dismiss, VISIBLE_MS);
  }, [clearDismiss, dismiss]);

  useEffect(() => {
    const unlisten = listen<SummaryPayload>("summary:show", (event) => {
      setData(event.payload);
      void overlay.setInteractive(SUMMARY_LABEL, true);
      scheduleDismiss();
    });
    // Cold-start handshake: if the window was created just now, ask the main
    // window to (re)send the latest summary in case its first emit raced our
    // listener registration.
    void emit("summary:request");
    return () => {
      void unlisten.then((fn) => fn());
      clearDismiss();
      if (copyTimer.current) window.clearTimeout(copyTimer.current);
    };
  }, [scheduleDismiss, clearDismiss]);

  const handleCopy = () => {
    if (!data) return;
    void navigator.clipboard.writeText(data.content).then(() => {
      setCopied(true);
      if (copyTimer.current) window.clearTimeout(copyTimer.current);
      copyTimer.current = window.setTimeout(() => setCopied(false), COPIED_VISIBLE_MS);
    });
  };

  const handleOpen = () => {
    void invoke("show_main").catch(() => {});
    void emit("navigate", "chat");
    dismiss();
  };

  return (
    <div className="fixed inset-x-0 top-0 flex justify-center pt-2">
      <AnimatePresence>
        {data ? (
          <motion.div
            key={data.ts}
            initial={{ opacity: 0, y: -14, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -14, scale: 0.96 }}
            transition={{ type: "spring", stiffness: 320, damping: 28 }}
            onPointerEnter={clearDismiss}
            onPointerLeave={scheduleDismiss}
            style={{ pointerEvents: "auto" }}
            className="w-[320px] overflow-hidden rounded-2xl border-[0.5px] border-acorn-brown/30 bg-card/95 shadow-lg backdrop-blur"
          >
            <div className="flex items-center gap-1.5 bg-gradient-to-r from-acorn-orange/90 to-acorn-brown px-3 py-1.5 text-acorn-paper">
              <MessageSquare className="h-3 w-3" />
              <span className="text-[11px] font-semibold">Acorn replied</span>
              {data.providerId ? (
                <span className="ml-auto font-mono text-[9px] uppercase tracking-wider opacity-80">
                  {data.providerId}
                </span>
              ) : null}
            </div>
            <div className="max-h-[110px] overflow-y-auto whitespace-pre-wrap px-3 py-2 text-[12px] leading-relaxed text-foreground">
              {data.preview}
            </div>
            <div className="flex justify-end gap-1 px-2 pb-1.5">
              <button
                type="button"
                onClick={handleCopy}
                className="flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium text-muted-foreground hover:bg-acorn-brown/10 hover:text-foreground"
              >
                {copied ? (
                  <Check className="h-3 w-3 text-acorn-olive" />
                ) : (
                  <Copy className="h-3 w-3" />
                )}
                {copied ? "Copied" : "Copy"}
              </button>
              <button
                type="button"
                onClick={handleOpen}
                className="rounded-full bg-acorn-orange/15 px-2 py-0.5 text-[10px] font-medium text-acorn-brown hover:bg-acorn-orange/25"
              >
                Open full
              </button>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
