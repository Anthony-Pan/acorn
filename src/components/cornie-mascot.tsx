import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { cn } from "@/lib/utils";
import { useChatStore } from "@/stores/chat";

interface CornieMascotProps {
  size?: number;
}

const STORAGE_KEY = "acorn:cornie-hidden";

type DragDropEvent = { type: string; paths?: string[] };

export function CornieMascot({ size = 48 }: CornieMascotProps) {
  const [hidden, setHidden] = useState(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) === "true";
    } catch {
      return false;
    }
  });
  const [chewing, setChewing] = useState(false);
  const chatPhase = useChatStore((s) => s.phase);
  const thinking = chatPhase === "thinking" || chatPhase === "tool" || chatPhase === "responding";

  useEffect(() => {
    if (hidden) return;
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onDragDropEvent((event) => {
      const payload = event.payload as DragDropEvent;
      if (payload.type !== "drop") return;
      const paths = payload.paths ?? [];
      if (paths.length === 0) return;
      setChewing(true);
      setTimeout(() => setChewing(false), 600);
      void navigator.clipboard
        .writeText(paths.join("\n"))
        .then(() => {
          const first = paths[0]?.split("/").pop() ?? paths[0];
          const summary =
            paths.length === 1
              ? `Cornie ate ${first}`
              : `Cornie ate ${paths.length} files (${first} + ${paths.length - 1} more)`;
          toast.success(summary, {
            description: "Paths copied to clipboard — paste into chat to share with Acorn.",
            id: "cornie-eats",
            duration: 6000,
          });
        })
        .catch((err: unknown) => {
          toast.error("Cornie spilled it", {
            description: err instanceof Error ? err.message : String(err),
          });
        });
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [hidden]);

  if (hidden) return null;

  const stemSize = Math.round(size * 0.24);
  const stemOffset = Math.round(size * 0.1);
  const eyeSize = Math.max(3, Math.round(size * 0.09));
  const eyeY = Math.round(size * 0.42);
  const eyeXOffset = Math.round(size * 0.18);
  const mouthSize = Math.round(size * 0.18);

  const handleHide = () => {
    try {
      localStorage.setItem(STORAGE_KEY, "true");
    } catch {}
    setHidden(true);
  };

  return (
    <button
      type="button"
      onClick={handleHide}
      aria-label="Hide Cornie"
      title="Click to hide Cornie"
      className={cn(
        "fixed bottom-5 right-5 z-30",
        chewing ? "cornie-chew" : thinking ? "cornie-think" : "cornie-bounce",
        "transition-transform duration-150 active:scale-90 hover:scale-105",
        "focus:outline-none",
      )}
      style={{ width: size, height: size + stemOffset }}
    >
      <div className="absolute inset-x-0 bottom-0" style={{ height: size }}>
        <div
          className="relative bg-acorn-brown w-full h-full"
          style={{ borderRadius: "50% 50% 45% 45% / 60% 60% 40% 40%" }}
        >
          <div
            className="absolute bg-acorn-brown-deep"
            style={{
              left: "50%",
              transform: "translateX(-50%)",
              top: -stemOffset,
              width: stemSize,
              height: stemSize,
              borderRadius: "40% 40% 50% 50%",
            }}
          />
          <div
            className="absolute bg-acorn-paper"
            style={{
              left: `calc(50% - ${eyeXOffset}px - ${eyeSize / 2}px)`,
              top: eyeY,
              width: eyeSize,
              height: eyeSize,
              borderRadius: "50%",
            }}
          />
          <div
            className="absolute bg-acorn-paper"
            style={{
              left: `calc(50% + ${eyeXOffset}px - ${eyeSize / 2}px)`,
              top: eyeY,
              width: eyeSize,
              height: eyeSize,
              borderRadius: "50%",
            }}
          />
          <div
            className="absolute border-b-2 border-acorn-paper"
            style={{
              left: `calc(50% - ${mouthSize / 2}px)`,
              top: eyeY + eyeSize + 4,
              width: mouthSize,
              height: mouthSize / 2,
              borderBottomLeftRadius: "100%",
              borderBottomRightRadius: "100%",
            }}
          />
        </div>
      </div>
      <style>{cornieKeyframes}</style>
    </button>
  );
}

const cornieKeyframes = `
@keyframes cornie-bounce {
  0%, 100% { transform: translateY(0); }
  50%      { transform: translateY(-4px); }
}
@keyframes cornie-chew {
  0%, 100% { transform: scale(1, 1)    translateY(0); }
  25%      { transform: scale(1.08, 0.92) translateY(2px); }
  50%      { transform: scale(0.92, 1.08) translateY(-3px); }
  75%      { transform: scale(1.05, 0.95) translateY(1px); }
}
@keyframes cornie-think {
  0%, 100% { transform: rotate(-3deg) translateY(0); }
  50%      { transform: rotate(3deg)  translateY(-2px); }
}
.cornie-bounce {
  animation: cornie-bounce 2.4s ease-in-out infinite;
}
.cornie-chew {
  animation: cornie-chew 0.55s ease-in-out;
}
.cornie-think {
  animation: cornie-think 1.1s ease-in-out infinite;
}
`;
