/**
 * Payload mirrored from the main window's chat store to overlay windows over the
 * `overlay:phase` Tauri event. Overlays run in their own webview process and
 * cannot read the Zustand store directly, so this event is their only source of
 * truth for chat activity.
 */
export interface OverlayPhase {
  phase: "idle" | "thinking" | "tool" | "responding";
  tool: string | null;
  toolCount: number;
  providerId: string | null;
}
