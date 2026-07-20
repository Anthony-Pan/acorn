import { cn } from "@/lib/utils";

interface VoiceHaloProps {
  visible: boolean;
}

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
@keyframes acorn-voice-halo-spin {
  to { transform: rotate(360deg); }
}
@keyframes acorn-voice-halo-breathe {
  0%, 100% { opacity: 0.55; }
  50% { opacity: 0.95; }
}
.voice-halo-frame {
  background: conic-gradient(
    from 0deg,
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
  animation: acorn-voice-halo-spin 12s linear infinite;
  filter: blur(2px);
}
.voice-halo-glow {
  box-shadow:
    inset 0 0 32px 4px rgba(255, 138, 76, 0.45),
    inset 0 0 96px 12px rgba(217, 70, 70, 0.18);
  animation: acorn-voice-halo-breathe 2.4s ease-in-out infinite;
}
`;
