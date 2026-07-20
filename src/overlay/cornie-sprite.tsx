import { cn } from "@/lib/utils";

export type CornieAnimation = "idle" | "bounce" | "think" | "chew" | "portalOut" | "portalIn";

interface CornieSpriteProps {
  size?: number;
  animation?: CornieAnimation;
  className?: string;
}

/**
 * Acorn's original pixel companion — a little acorn sprite drawn with rounded
 * shapes in the autumn palette. Shared by the desktop companion overlay and any
 * in-app cameo. All motion is CSS keyframes (GPU-composited); `idle` applies no
 * animation class so a resting sprite costs nothing to repaint.
 */
export function CornieSprite({ size = 48, animation = "bounce", className }: CornieSpriteProps) {
  const stemSize = Math.round(size * 0.24);
  const stemOffset = Math.round(size * 0.1);
  const eyeSize = Math.max(3, Math.round(size * 0.09));
  const eyeY = Math.round(size * 0.42);
  const eyeXOffset = Math.round(size * 0.18);
  const mouthSize = Math.round(size * 0.18);

  const animClass =
    animation === "chew"
      ? "cornie-chew"
      : animation === "think"
        ? "cornie-think"
        : animation === "bounce"
          ? "cornie-bounce"
          : animation === "portalOut"
            ? "cornie-portal-out"
            : animation === "portalIn"
              ? "cornie-portal-in"
              : undefined;

  return (
    <div
      className={cn("relative", animClass, className)}
      style={{ width: size, height: size + stemOffset }}
    >
      <div className="absolute inset-x-0 bottom-0" style={{ height: size }}>
        <div
          className="relative h-full w-full bg-acorn-brown"
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
            className="absolute bg-acorn-paper cornie-eye"
            style={{
              left: `calc(50% - ${eyeXOffset}px - ${eyeSize / 2}px)`,
              top: eyeY,
              width: eyeSize,
              height: eyeSize,
              borderRadius: "50%",
            }}
          />
          <div
            className="absolute bg-acorn-paper cornie-eye"
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
    </div>
  );
}

const cornieKeyframes = `
@keyframes cornie-bounce {
  0%, 100% { transform: translateY(0); }
  50%      { transform: translateY(-4px); }
}
@keyframes cornie-chew {
  0%, 100% { transform: scale(1, 1)     translateY(0); }
  25%      { transform: scale(1.08, 0.92) translateY(2px); }
  50%      { transform: scale(0.92, 1.08) translateY(-3px); }
  75%      { transform: scale(1.05, 0.95) translateY(1px); }
}
@keyframes cornie-think {
  0%, 100% { transform: rotate(-3deg) translateY(0); }
  50%      { transform: rotate(3deg)  translateY(-2px); }
}
@keyframes cornie-blink {
  0%, 92%, 100% { transform: scaleY(1); }
  96%           { transform: scaleY(0.1); }
}
@keyframes cornie-portal-out {
  0%   { transform: scale(1) rotate(0);       opacity: 1; }
  100% { transform: scale(0.2) rotate(180deg); opacity: 0; }
}
@keyframes cornie-portal-in {
  0%   { transform: scale(0.2) rotate(-180deg); opacity: 0; }
  100% { transform: scale(1) rotate(0);         opacity: 1; }
}
.cornie-bounce { animation: cornie-bounce 2.4s ease-in-out infinite; }
.cornie-chew { animation: cornie-chew 0.55s ease-in-out; }
.cornie-think { animation: cornie-think 1.1s ease-in-out infinite; }
.cornie-portal-out { animation: cornie-portal-out 0.18s ease-in forwards; }
.cornie-portal-in { animation: cornie-portal-in 0.22s ease-out; }
.cornie-eye { animation: cornie-blink 5.2s ease-in-out infinite; transform-origin: center; }
`;
