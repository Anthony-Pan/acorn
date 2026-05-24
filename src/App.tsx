import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface UpdateAvailable {
  currentVersion: string;
  latestVersion: string;
  releaseName: string | null;
  releaseNotes: string | null;
  releaseUrl: string;
}

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
    const timer = window.setTimeout(async () => {
      try {
        const update = await invoke<UpdateAvailable | null>("check_for_update");
        if (!update) return;
        toast.message(`Acorn ${update.latestVersion} is available`, {
          description: `You're on ${update.currentVersion}. Open the release page to download.`,
          duration: 12000,
          action: {
            label: "Open release",
            onClick: () => {
              window.open(update.releaseUrl, "_blank", "noreferrer noopener");
            },
          },
        });
      } catch {}
    }, 60_000);
    return () => window.clearTimeout(timer);
  }, []);

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
