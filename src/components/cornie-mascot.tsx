import { useState } from "react";

import { cn } from "@/lib/utils";

interface CornieMascotProps {
  size?: number;
}

const STORAGE_KEY = "acorn:cornie-hidden";

export function CornieMascot({ size = 48 }: CornieMascotProps) {
  const [hidden, setHidden] = useState(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) === "true";
    } catch {
      return false;
    }
  });

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
        "fixed bottom-5 right-5 z-30 cornie-bounce",
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
.cornie-bounce {
  animation: cornie-bounce 2.4s ease-in-out infinite;
}
`;
