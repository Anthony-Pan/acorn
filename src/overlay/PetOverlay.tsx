import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { type PetCorner, pet } from "@/lib/pet";
import type { OverlayPhase } from "@/types/overlay";
import { type CornieAnimation, CornieSprite } from "./cornie-sprite";

const CORNERS: PetCorner[] = ["bottomRight", "bottomLeft", "topLeft", "topRight"];
const CHEW_MS = 600;
const PORTAL_OUT_MS = 200;
const PORTAL_IN_MS = 240;

type DragDropPayload = { type: string; paths?: string[] };

/**
 * The desktop companion in its own always-on-top corner window. It reacts to
 * chat activity, eats files dropped onto it (routing their paths to the main
 * window), and hops between corners with a portal swirl when clicked. Idle motion
 * is a gentle GPU-composited bounce, so a resting companion is cheap.
 */
export function PetOverlay() {
  const [phase, setPhase] = useState<OverlayPhase["phase"]>("idle");
  const [chewing, setChewing] = useState(false);
  const [portal, setPortal] = useState<"out" | "in" | null>(null);
  const cornerIndex = useRef(0);
  const teleporting = useRef(false);
  const chewTimer = useRef<number | null>(null);

  useEffect(() => {
    const unlisten = listen<OverlayPhase>("overlay:phase", (event) => {
      setPhase(event.payload.phase);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const win = getCurrentWindow();
    const unlisten = win.onDragDropEvent((event) => {
      const payload = event.payload as DragDropPayload;
      if (payload.type !== "drop") return;
      const paths = payload.paths ?? [];
      if (paths.length === 0) return;
      setChewing(true);
      if (chewTimer.current) window.clearTimeout(chewTimer.current);
      chewTimer.current = window.setTimeout(() => setChewing(false), CHEW_MS);
      void emit("pet:files-dropped", { paths });
    });
    return () => {
      void unlisten.then((fn) => fn());
      if (chewTimer.current) window.clearTimeout(chewTimer.current);
    };
  }, []);

  const handleClick = () => {
    if (teleporting.current) return;
    teleporting.current = true;
    setPortal("out");
    window.setTimeout(() => {
      cornerIndex.current = (cornerIndex.current + 1) % CORNERS.length;
      void pet.teleport(CORNERS[cornerIndex.current] ?? "bottomRight").catch(() => {});
      setPortal("in");
      window.setTimeout(() => {
        setPortal(null);
        teleporting.current = false;
      }, PORTAL_IN_MS);
    }, PORTAL_OUT_MS);
  };

  const animation: CornieAnimation =
    portal === "out"
      ? "portalOut"
      : portal === "in"
        ? "portalIn"
        : chewing
          ? "chew"
          : phase === "thinking" || phase === "tool" || phase === "responding"
            ? "think"
            : "bounce";

  return (
    <div className="flex h-screen w-screen items-center justify-center">
      <button
        type="button"
        onClick={handleClick}
        aria-label="Cornie — click to hop to another corner"
        title="Cornie"
        className="cursor-pointer border-0 bg-transparent p-0 focus:outline-none"
        style={{ pointerEvents: "auto" }}
      >
        <CornieSprite size={64} animation={animation} />
      </button>
    </div>
  );
}
