import { Loader2, Wrench } from "lucide-react";

import { cn } from "@/lib/utils";
import { useChatStore } from "@/stores/chat";
import { useProvidersStore } from "@/stores/providers";
import { useSettingsStore } from "@/stores/settings";

export function StatusCapsule() {
  const phase = useChatStore((s) => s.phase);
  const activeTools = useChatStore((s) => s.activeTools);
  const current = useChatStore((s) => s.current);
  const catalog = useProvidersStore((s) => s.catalog);
  const activeId = useSettingsStore((s) => s.activeProviderId);

  const conversationProviderId = current?.providerId ?? activeId ?? null;
  const conversationProvider = catalog.find((p) => p.id === conversationProviderId) ?? null;

  if (phase === "idle" && activeTools.length === 0) return null;

  const label =
    phase === "thinking"
      ? "Thinking"
      : phase === "tool"
        ? `Running ${activeTools[0]?.name ?? "tool"}`
        : phase === "responding"
          ? "Replying"
          : "Working";

  return (
    <div
      aria-live="polite"
      className={cn(
        "fixed top-3 left-1/2 -translate-x-1/2 z-40",
        "px-3 py-1 rounded-full border-[0.5px] bg-card/95 backdrop-blur",
        "border-acorn-orange/40 shadow-sm",
        "flex items-center gap-2 text-[11px]",
      )}
    >
      {phase === "tool" ? (
        <Wrench className="w-3 h-3 text-acorn-orange" />
      ) : (
        <Loader2 className="w-3 h-3 animate-spin text-acorn-orange" />
      )}
      <span className="font-medium text-foreground">{label}</span>
      {conversationProvider ? (
        <>
          <span className="text-muted-foreground/60">·</span>
          <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-wider">
            {conversationProvider.id}
          </span>
        </>
      ) : null}
      {activeTools.length > 1 ? (
        <>
          <span className="text-muted-foreground/60">·</span>
          <span className="text-muted-foreground">+{activeTools.length - 1}</span>
        </>
      ) : null}
    </div>
  );
}
