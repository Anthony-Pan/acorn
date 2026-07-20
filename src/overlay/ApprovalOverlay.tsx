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
            transition={{ duration: 0.2, ease: [0.25, 0.1, 0.25, 1] }}
            style={{ pointerEvents: "auto" }}
            className="glass w-[340px] overflow-hidden rounded-lg"
          >
            <div className="flex items-center gap-1.5 border-b-[0.5px] border-border px-3 py-1.5">
              <ShieldQuestion className="h-3.5 w-3.5 text-primary" />
              <span className="text-[11px] font-semibold text-foreground">Allow this tool?</span>
              <span className="ml-auto font-mono text-[11px] text-muted-foreground">
                {request.toolName}
              </span>
            </div>
            <div className="max-h-[64px] overflow-y-auto px-3 py-2 font-mono text-[11px] leading-relaxed text-muted-foreground">
              {request.argsPreview || "(no arguments)"}
            </div>
            <div className="flex justify-end gap-2 px-2 pb-2">
              <button
                type="button"
                onClick={() => respond(false)}
                className="rounded-full px-3 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
              >
                Deny
              </button>
              <button
                type="button"
                onClick={() => respond(true)}
                className="rounded-full bg-primary px-3 py-1 text-[11px] font-medium text-primary-foreground transition-colors hover:bg-primary/90"
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
