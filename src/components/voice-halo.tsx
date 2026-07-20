import { cn } from "@/lib/utils";

interface VoiceHaloProps {
  visible: boolean;
}

/**
 * Full-viewport edge glow shown while the mic is live: a warm conic gradient
 * sweeping around a masked border band, over a breathing inner bloom. The sweep
 * animates the gradient's start angle (a registered custom property) rather
 * than rotating the element, so the band hugs the window edges at any aspect
 * ratio; without `@property` support it degrades to a static frame.
 */
export function VoiceHalo({ visible }: VoiceHaloProps) {
  return (
    <div
      aria-hidden
      className={cn(
        "fixed inset-0 pointer-events-none z-50 transition-opacity",
        visible ? "opacity-100 duration-300" : "opacity-0 duration-500",
      )}
    >
      <div className="absolute inset-0 voice-halo-frame" />
      <div className="absolute inset-0 voice-halo-glow" />
      <style>{voiceHaloKeyframes}</style>
    </div>
  );
}

const voiceHaloKeyframes = `
@property --acorn-halo-angle {
  syntax: "<angle>";
  inherits: false;
  initial-value: 0deg;
}
@keyframes acorn-voice-halo-sweep {
  to { --acorn-halo-angle: 360deg; }
}
@keyframes acorn-voice-halo-breathe {
  0%, 100% { opacity: 0.55; }
  50% { opacity: 0.95; }
}
.voice-halo-frame {
  background: conic-gradient(
    from var(--acorn-halo-angle),
    rgba(255, 138, 76, 0.85),
    rgba(217, 70, 70, 0.82),
    rgba(168, 92, 50, 0.7),
    rgba(196, 134, 62, 0.78),
    rgba(255, 191, 105, 0.85),
    rgba(255, 138, 76, 0.85)
  );
  -webkit-mask:
    linear-gradient(#000, #000) content-box,
    linear-gradient(#000, #000);
  -webkit-mask-composite: xor;
  mask:
    linear-gradient(#000, #000) content-box,
    linear-gradient(#000, #000);
  mask-composite: exclude;
  padding: 6px;
  animation: acorn-voice-halo-sweep 8s linear infinite;
  filter: blur(2px);
}
.voice-halo-glow {
  box-shadow:
    inset 0 0 32px 4px rgba(255, 138, 76, 0.45),
    inset 0 0 96px 12px rgba(217, 70, 70, 0.18);
  animation: acorn-voice-halo-breathe 2.4s ease-in-out infinite;
}
@media (prefers-reduced-motion: reduce) {
  .voice-halo-frame,
  .voice-halo-glow {
    animation: none;
  }
}
`;
