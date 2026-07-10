import { getCurrentWindow } from "@tauri-apps/api/window";
import { Check, Copy, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { pins } from "@/lib/pin";

interface PinOverlayProps {
  pinId: string;
}

const POSITION_DEBOUNCE_MS = 250;
const COPIED_VISIBLE_MS = 1500;

/**
 * An individual pinned reply, living in its own always-on-top desktop card
 * window. Loads its content by id, persists its position when dragged, and
 * removes both the window and the row when dismissed.
 */
export function PinOverlay({ pinId }: PinOverlayProps) {
  const [label, setLabel] = useState("Pinned");
  const [content, setContent] = useState("");
  const [copied, setCopied] = useState(false);
  const copyTimer = useRef<number | null>(null);

  useEffect(() => {
    if (!pinId) {
      void getCurrentWindow().close();
      return;
    }
    let active = true;
    void pins
      .get(pinId)
      .then((pin) => {
        if (!active) return;
        setLabel(pin.label);
        setContent(pin.content);
      })
      .catch(() => {
        void getCurrentWindow().close();
      });
    return () => {
      active = false;
    };
  }, [pinId]);

  // Persist the card's logical position whenever the user drags it.
  useEffect(() => {
    if (!pinId) return;
    const win = getCurrentWindow();
    let moveTimer: number | null = null;
    const unlistenPromise = win.onMoved(({ payload }) => {
      if (moveTimer) window.clearTimeout(moveTimer);
      moveTimer = window.setTimeout(() => {
        // Read the scale factor at save time: the drag may have ended on a
        // monitor with a different DPI than the one it started on.
        void win
          .scaleFactor()
          .then((factor) => pins.updatePosition(pinId, payload.x / factor, payload.y / factor))
          .catch(() => {});
      }, POSITION_DEBOUNCE_MS);
    });
    return () => {
      if (moveTimer) window.clearTimeout(moveTimer);
      void unlistenPromise.then((fn) => fn());
    };
  }, [pinId]);

  useEffect(
    () => () => {
      if (copyTimer.current) window.clearTimeout(copyTimer.current);
    },
    [],
  );

  const handleCopy = () => {
    void navigator.clipboard.writeText(content).then(() => {
      setCopied(true);
      if (copyTimer.current) window.clearTimeout(copyTimer.current);
      copyTimer.current = window.setTimeout(() => setCopied(false), COPIED_VISIBLE_MS);
    });
  };

  const handleClose = () => {
    void pins.delete(pinId).finally(() => {
      void getCurrentWindow().close();
    });
  };

  return (
    <div className="h-screen w-screen overflow-hidden p-1.5" style={{ pointerEvents: "auto" }}>
      <div className="flex h-full flex-col overflow-hidden rounded-2xl border-[0.5px] border-acorn-brown/30 bg-card/95 shadow-lg backdrop-blur">
        <div className="h-1 bg-gradient-to-r from-acorn-orange to-acorn-brown" />
        <div
          data-tauri-drag-region
          className="flex cursor-grab items-center justify-between gap-2 px-3 pt-1.5 pb-1 active:cursor-grabbing"
        >
          <span
            data-tauri-drag-region
            className="truncate text-[11px] font-semibold text-acorn-brown"
          >
            {label}
          </span>
          <button
            type="button"
            onClick={handleClose}
            aria-label="Remove pin"
            className="rounded-full p-0.5 text-muted-foreground hover:bg-acorn-brown/10 hover:text-foreground"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto whitespace-pre-wrap px-3 text-[12px] leading-relaxed text-foreground">
          {content}
        </div>
        <div className="flex justify-end px-2 pt-1 pb-1.5">
          <button
            type="button"
            onClick={handleCopy}
            className="flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium text-muted-foreground hover:bg-acorn-brown/10 hover:text-foreground"
          >
            {copied ? <Check className="h-3 w-3 text-acorn-olive" /> : <Copy className="h-3 w-3" />}
            {copied ? "Copied" : "Copy"}
          </button>
        </div>
      </div>
    </div>
  );
}
