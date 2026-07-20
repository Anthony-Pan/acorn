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
            transition={{ duration: 0.2, ease: [0.25, 0.1, 0.25, 1] }}
            onPointerEnter={clearDismiss}
            onPointerLeave={scheduleDismiss}
            style={{ pointerEvents: "auto" }}
            className="glass w-[320px] overflow-hidden rounded-lg"
          >
            <div className="flex items-center gap-1.5 border-b-[0.5px] border-border px-3 py-1.5">
              <MessageSquare className="h-3 w-3 text-primary" />
              <span className="text-[11px] font-semibold text-foreground">Acorn replied</span>
              {data.providerId ? (
                <span className="ml-auto font-mono text-[11px] uppercase tracking-wider text-muted-foreground">
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
                className="flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
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
                className="rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground transition-colors hover:bg-accent"
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
