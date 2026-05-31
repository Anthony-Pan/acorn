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
  model: string | null;
  /** Short preview of the latest assistant reply, for the hover-expanded capsule. */
  lastReply: string | null;
  error: string | null;
}

/** Voice capture state mirrored to the island over the `overlay:voice` event. */
export interface OverlayVoice {
  state: "idle" | "recording" | "transcribing";
}

/** Payload for the transient reply-summary overlay (`summary:show` event). */
export interface SummaryPayload {
  content: string;
  preview: string;
  providerId: string | null;
  ts: number;
}
