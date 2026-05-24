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
import { useEffect, useRef, useState } from "react";
import { Toaster, toast } from "sonner";
import { AcornStash } from "@/components/acorn-stash";
import { CalendarView } from "@/components/calendar-view";
import { CanvasView } from "@/components/canvas-view";
import { ChatView } from "@/components/chat-view";
import { CornieMascot } from "@/components/cornie-mascot";
import { SettingsPage } from "@/components/settings/settings-page";
import { StatusCapsule } from "@/components/status-capsule";
import { TaskInput } from "@/components/task-input";
import { activity } from "@/lib/activity";
import { useChatStore } from "@/stores/chat";
import { useProvidersStore } from "@/stores/providers";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";

type View = "input" | "stash" | "chat" | "settings" | "calendar" | "canvas";

function App() {
  const hydrateSession = useSessionStore((s) => s.hydrate);
  const hydrateSettings = useSettingsStore((s) => s.hydrate);
  const hydrateProviders = useProvidersStore((s) => s.hydrate);
  const hydrateChat = useChatStore((s) => s.hydrate);
  const current = useSessionStore((s) => s.current);
  const isStashing = useSessionStore((s) => s.isStashing);
  const chatPhase = useChatStore((s) => s.phase);
  const chatCurrent = useChatStore((s) => s.current);

  const [view, setView] = useState<View>("input");
  const previousChatPhase = useRef<typeof chatPhase>("idle");

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
    const LAST_BRIEF_KEY = "acorn:last-brief";
    const MIN_INTERVAL_MS = 18 * 60 * 60 * 1000;
    const timer = window.setTimeout(async () => {
      try {
        const lastBrief = Number.parseInt(localStorage.getItem(LAST_BRIEF_KEY) ?? "0", 10);
        if (Number.isFinite(lastBrief) && Date.now() - lastBrief < MIN_INTERVAL_MS) return;
        const brief = await activity.dailyBrief(24);
        if (brief.totalEntries === 0) return;
        localStorage.setItem(LAST_BRIEF_KEY, String(Date.now()));
        toast.message("Yesterday at a glance", {
          description: brief.summary,
          duration: 10000,
        });
      } catch {}
    }, 6_000);
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
      } else if (url.host === "canvas") {
        setView("canvas");
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
    if (view === "settings" || view === "chat" || view === "calendar" || view === "canvas") return;
    if (current && current.tasks.length > 0) {
      setView("stash");
    } else if (!isStashing) {
      setView("input");
    }
  }, [current, isStashing, view]);

  useEffect(() => {
    const wasResponding =
      previousChatPhase.current === "responding" || previousChatPhase.current === "thinking";
    previousChatPhase.current = chatPhase;
    if (chatPhase !== "idle" || !wasResponding) return;
    if (view === "chat" && !document.hidden) return;
    const lastReply = chatCurrent?.messages
      .filter((m) => m.role === "assistant" && m.content.trim().length > 0)
      .at(-1);
    if (!lastReply) return;
    const preview =
      lastReply.content.length > 140
        ? `${lastReply.content.slice(0, 140).trim()}…`
        : lastReply.content.trim();
    toast.message("Acorn replied", {
      description: preview,
      duration: 8000,
      action: {
        label: "Open",
        onClick: () => setView("chat"),
      },
    });
  }, [chatPhase, chatCurrent, view]);

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
          ) : view === "canvas" ? (
            <CanvasView onBack={() => setView(defaultView)} />
          ) : view === "stash" ? (
            <AcornStash
              onOpenSettings={() => setView("settings")}
              onOpenChat={() => setView("chat")}
              onOpenCalendar={() => setView("calendar")}
              onOpenCanvas={() => setView("canvas")}
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
              onOpenCanvas={() => setView("canvas")}
            />
          )}
        </motion.div>
      </AnimatePresence>
      <CornieMascot />
      <StatusCapsule />
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
