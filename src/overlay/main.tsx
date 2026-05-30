import ReactDOM from "react-dom/client";

import "../index.css";
import { NotchOverlay } from "./NotchOverlay";

/**
 * Single entry point for every overlay window. The `?kind=` query string (set by
 * the Rust window builder) selects which surface renders, so one `overlay.html`
 * bundle serves the notch capsule, pins, the companion, and reply summaries.
 *
 * No `<React.StrictMode>`: overlays own `listen()` subscriptions and pointer →
 * IPC toggles, and StrictMode's double-invoked effects would double-register
 * listeners and double-fire the interactivity IPC.
 */
const params = new URLSearchParams(window.location.search);
const kind = params.get("kind") ?? "notch";

const root = document.getElementById("root");
if (!root) {
  throw new Error("overlay: missing #root element");
}

function renderOverlay(which: string) {
  switch (which) {
    // Additional overlay kinds (pin, pet, summary) are wired in as they land.
    default:
      return <NotchOverlay />;
  }
}

ReactDOM.createRoot(root).render(renderOverlay(kind));
