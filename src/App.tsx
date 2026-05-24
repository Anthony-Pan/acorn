import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";
import { Toaster, toast } from "sonner";

import { AcornStash } from "@/components/acorn-stash";
import { CalendarView } from "@/components/calendar-view";
import { ChatView } from "@/components/chat-view";
import { SettingsPage } from "@/components/settings/settings-page";
import { TaskInput } from "@/components/task-input";
import { useChatStore } from "@/stores/chat";
import { useProvidersStore } from "@/stores/providers";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";

type View = "input" | "stash" | "chat" | "settings" | "calendar";

function App() {
  const hydrateSession = useSessionStore((s) => s.hydrate);
  const hydrateSettings = useSettingsStore((s) => s.hydrate);
  const hydrateProviders = useProvidersStore((s) => s.hydrate);
  const hydrateChat = useChatStore((s) => s.hydrate);
  const current = useSessionStore((s) => s.current);
  const isStashing = useSessionStore((s) => s.isStashing);

  const [view, setView] = useState<View>("input");

  useEffect(() => {
    void Promise.all([hydrateSettings(), hydrateProviders(), hydrateSession(), hydrateChat()]);
  }, [hydrateSession, hydrateSettings, hydrateProviders, hydrateChat]);

  useEffect(() => {
    const unlistenSubmit = listen<{ text: string }>("quick:submit", async (event) => {
      const { activeProviderId, language } = useSettingsStore.getState();
      const window = getCurrentWindow();
      await window.show();
      await window.setFocus();
      if (!activeProviderId) {
        setView("settings");
        return;
      }
      try {
        await useSessionStore.getState().stash(event.payload.text, activeProviderId, language);
      } catch (err) {
        console.error("quick stash failed", err);
      }
    });

    const unlistenNavigate = listen<string>("navigate", (event) => {
      if (event.payload === "settings") setView("settings");
    });

    const unlistenPinShortcut = listen("shortcut:pin-response", async () => {
      const main = getCurrentWindow();
      await main.show();
      await main.setFocus();
      toast.message("Pin to desktop", {
        description: "Pinning replies to the desktop lands in an upcoming Acorn release.",
        id: "shortcut-pin",
      });
    });

    const unlistenScreenshotShortcut = listen("shortcut:screenshot", async () => {
      try {
        const savedPath = await invoke<string>("capture_primary_screen");
        const main = getCurrentWindow();
        await main.show();
        await main.setFocus();
        try {
          await navigator.clipboard.writeText(savedPath);
        } catch {}
        const filename = savedPath.split("/").pop() ?? savedPath;
        toast.success("Screenshot saved", {
          description: `${filename} — path copied to clipboard.`,
          id: "shortcut-screenshot",
          duration: 8000,
          action: {
            label: "Reveal",
            onClick: async () => {
              try {
                await invoke("plugin:opener|reveal_item_in_dir", { path: savedPath });
              } catch (err) {
                console.error("reveal failed", err);
              }
            },
          },
        });
      } catch (err) {
        toast.error("Screenshot failed", {
          description: err instanceof Error ? err.message : String(err),
          id: "shortcut-screenshot-error",
        });
      }
    });

    const unlistenPushToTalkShortcut = listen("shortcut:push-to-talk", async () => {
      const main = getCurrentWindow();
      await main.show();
      await main.setFocus();
      toast.message("Push to talk", {
        description: "Hold-to-talk voice input ships with the next voice overhaul.",
        id: "shortcut-ptt",
      });
    });

    const unlistenQuickAskShortcut = listen("shortcut:quick-ask", async () => {
      const window = getCurrentWindow();
      await window.show();
      await window.setFocus();
      toast.message("Quick ask", {
        description: "A Spotlight-style ask overlay is coming in a future release.",
        id: "shortcut-quick-ask",
      });
    });

    const unlistenDeepLink = listen<string>("deep-link", async (event) => {
      const url = parseDeepLink(event.payload);
      if (!url) return;
      const main = getCurrentWindow();
      await main.show();
      await main.setFocus();
      if (url.host === "stash" && url.text) {
        sessionStorage.setItem("acorn:prefill", url.text);
        setView("input");
        toast.message("Imported from link", {
          description: "Review the text and tap Stash it 🌰",
          id: "deep-link-prefill",
        });
      } else if (url.host === "settings") {
        setView("settings");
      } else if (url.host === "calendar") {
        setView("calendar");
      } else if (url.host === "chat") {
        setView("chat");
      }
    });

    return () => {
      void unlistenSubmit.then((fn) => fn());
      void unlistenNavigate.then((fn) => fn());
      void unlistenDeepLink.then((fn) => fn());
      void unlistenPinShortcut.then((fn) => fn());
      void unlistenScreenshotShortcut.then((fn) => fn());
      void unlistenPushToTalkShortcut.then((fn) => fn());
      void unlistenQuickAskShortcut.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (view === "settings" || view === "chat" || view === "calendar") return;
    if (current && current.tasks.length > 0) {
      setView("stash");
    } else if (!isStashing) {
      setView("input");
    }
  }, [current, isStashing, view]);

  const defaultView = current && current.tasks.length > 0 ? "stash" : "input";

  return (
    <>
      <AnimatePresence mode="wait">
        <motion.div
          key={view}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.18 }}
        >
          {view === "settings" ? (
            <SettingsPage onClose={() => setView(defaultView)} />
          ) : view === "chat" ? (
            <ChatView onBack={() => setView(defaultView)} />
          ) : view === "calendar" ? (
            <CalendarView onBack={() => setView(defaultView)} />
          ) : view === "stash" ? (
            <AcornStash
              onOpenSettings={() => setView("settings")}
              onOpenChat={() => setView("chat")}
              onOpenCalendar={() => setView("calendar")}
              onAddMore={() => {
                useSessionStore.getState().startNew();
                setView("input");
              }}
            />
          ) : (
            <TaskInput
              onOpenSettings={() => setView("settings")}
              onOpenChat={() => setView("chat")}
              onOpenCalendar={() => setView("calendar")}
            />
          )}
        </motion.div>
      </AnimatePresence>
      <Toaster
        position="top-center"
        toastOptions={{
          style: {
            background: "var(--acorn-paper)",
            color: "var(--acorn-ink)",
            border: "0.5px solid rgba(139, 69, 19, 0.18)",
            fontSize: "13px",
          },
        }}
      />
    </>
  );
}

export default App;

interface ParsedDeepLink {
  host: string;
  text: string | null;
}

function parseDeepLink(raw: string): ParsedDeepLink | null {
  try {
    const url = new URL(raw);
    if (url.protocol !== "acorn:") return null;
    const host = url.hostname || url.pathname.replace(/^\/+/, "").split("/")[0] || "";
    return { host, text: url.searchParams.get("text") };
  } catch {
    return null;
  }
}
