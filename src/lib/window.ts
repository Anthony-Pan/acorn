import { invoke } from "@tauri-apps/api/core";

/**
 * Typed wrappers for the desktop overlay windows (notch capsule, pins, the
 * companion, reply summaries). Components call these instead of `invoke`
 * directly, mirroring the rest of `src/lib/*`.
 */
export const overlay = {
  /** Toggle whether an overlay window swallows pointer events. */
  setInteractive: (label: string, interactive: boolean) =>
    invoke<void>("set_overlay_interactive", { label, interactive }),
  showNotch: () => invoke<void>("show_notch_overlay"),
  hideNotch: () => invoke<void>("hide_notch_overlay"),
  showSummary: () => invoke<void>("show_summary_overlay"),
  hideSummary: () => invoke<void>("hide_summary_overlay"),
};
