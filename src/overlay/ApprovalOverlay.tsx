import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { ShieldQuestion } from "lucide-react";
import { useEffect, useState } from "react";

import { overlay } from "@/lib/window";

interface ApprovalRequest {
  requestId: string;
  toolName: string;
  argsPreview: string;
}

/**
 * The tool-permission card that drops just below the dynamic island when a tool
 * wants to act on the machine. It shows the tool and its arguments and lets the
 * user Deny or Allow; the decision is sent straight back to the approval broker.
 * Lives in its own overlay window (created hidden at boot so this listener is
 * always ready).
 */
export function ApprovalOverlay() {
  const [request, setRequest] = useState<ApprovalRequest | null>(null);

  useEffect(() => {
    const unlisten = listen<ApprovalRequest>("approval:show", (event) => {
      setRequest(event.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const respond = (allow: boolean) => {
    if (request) {
      void invoke("respond_tool_approval", { requestId: request.requestId, allow }).catch(() => {});
    }
    setRequest(null);
    void overlay.hideApproval();
  };

  return (
    <div className="fixed inset-x-0 top-0 flex justify-center pt-1">
      <AnimatePresence>
        {request ? (
          <motion.div
            key={request.requestId}
            initial={{ opacity: 0, y: -12, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -12, scale: 0.96 }}
            transition={{ type: "spring", stiffness: 340, damping: 28 }}
            style={{ pointerEvents: "auto" }}
            className="w-[340px] overflow-hidden rounded-2xl border-[0.5px] border-acorn-brown/30 bg-card/95 shadow-lg backdrop-blur"
          >
            <div className="flex items-center gap-1.5 bg-gradient-to-r from-acorn-orange/90 to-acorn-brown px-3 py-1.5 text-acorn-paper">
              <ShieldQuestion className="h-3.5 w-3.5" />
              <span className="text-[11px] font-semibold">Allow this tool?</span>
              <span className="ml-auto font-mono text-[10px] opacity-90">{request.toolName}</span>
            </div>
            <div className="max-h-[64px] overflow-y-auto px-3 py-2 font-mono text-[11px] leading-relaxed text-muted-foreground">
              {request.argsPreview || "(no arguments)"}
            </div>
            <div className="flex justify-end gap-2 px-2 pb-2">
              <button
                type="button"
                onClick={() => respond(false)}
                className="rounded-full px-3 py-1 text-[11px] font-medium text-muted-foreground hover:bg-acorn-red/10 hover:text-acorn-red"
              >
                Deny
              </button>
              <button
                type="button"
                onClick={() => respond(true)}
                className="rounded-full bg-acorn-orange px-3 py-1 text-[11px] font-medium text-acorn-paper hover:bg-acorn-brown"
              >
                Allow
              </button>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
