import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
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
import { SettingsPage } from "@/components/settings/settings-page";
import { TaskInput } from "@/components/task-input";
import { activity } from "@/lib/activity";
import { pins } from "@/lib/pin";
import { overlay } from "@/lib/window";
import { useChatStore } from "@/stores/chat";
import { useProvidersStore } from "@/stores/providers";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";
import type { SummaryPayload } from "@/types/overlay";

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
  const latestSummary = useRef<SummaryPayload | null>(null);

  useEffect(() => {
    void Promise.all([hydrateSettings(), hydrateProviders(), hydrateSession(), hydrateChat()]);
  }, [hydrateSession, hydrateSettings, hydrateProviders, hydrateChat]);

  // Mirror chat activity to the overlay windows (notch capsule, companion).
  // Overlays live in their own webview process and cannot read the Zustand
  // store, so this event is their only source of truth.
  useEffect(() => {
    let last = "";
    const broadcast = (s: ReturnType<typeof useChatStore.getState>) => {
      const payload = {
        phase: s.phase,
        tool: s.activeTools[0]?.name ?? null,
        toolCount: s.activeTools.length,
        providerId: s.current?.providerId ?? null,
      };
      const key = JSON.stringify(payload);
      if (key === last) return;
      last = key;
      void emit("overlay:phase", payload);
    };
    broadcast(useChatStore.getState());
    return useChatStore.subscribe(broadcast);
  }, []);

  // Files dropped on the main window — or onto the companion (which forwards
  // their paths) — are copied to the clipboard for pasting into a chat.
  useEffect(() => {
    const eatFiles = (paths: string[]) => {
      const valid = paths.filter((p) => p.length > 0);
      if (valid.length === 0) return;
      void navigator.clipboard
        .writeText(valid.join("\n"))
        .then(() => {
          const first = valid[0]?.split("/").pop() ?? valid[0];
          const summary =
            valid.length === 1
              ? `Cornie ate ${first}`
              : `Cornie ate ${valid.length} files (${first} + ${valid.length - 1} more)`;
          toast.success(summary, {
            description: "Paths copied to clipboard — paste into chat to share with Acorn.",
            id: "cornie-eats",
            duration: 6000,
          });
        })
        .catch((err: unknown) => {
          toast.error("Cornie spilled it", {
            description: err instanceof Error ? err.message : String(err),
          });
        });
    };

    const unlistenMainDrop = getCurrentWindow().onDragDropEvent((event) => {
      const payload = event.payload as { type: string; paths?: string[] };
      if (payload.type !== "drop") return;
      eatFiles(payload.paths ?? []);
    });
    const unlistenPetDrop = listen<{ paths: string[] }>("pet:files-dropped", (event) => {
      eatFiles(event.payload.paths ?? []);
    });
    return () => {
      void unlistenMainDrop.then((fn) => fn());
      void unlistenPetDrop.then((fn) => fn());
    };
  }, []);

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
    const LAST_RECAP_KEY = "acorn:last-recap";
    const DAY_MS = 24 * 60 * 60 * 1000;
    const briefIntervalMs = 18 * 60 * 60 * 1000;
    const recapIntervalMs = 7 * DAY_MS;

    const timer = window.setTimeout(async () => {
      try {
        const lastBrief = Number.parseInt(localStorage.getItem(LAST_BRIEF_KEY) ?? "0", 10);
        if (!Number.isFinite(lastBrief) || Date.now() - lastBrief >= briefIntervalMs) {
          const brief = await activity.dailyBrief(24);
          if (brief.totalEntries > 0) {
            localStorage.setItem(LAST_BRIEF_KEY, String(Date.now()));
            toast.message("Yesterday at a glance", {
              description: brief.summary,
              duration: 10000,
            });
          }
        }

        const lastRecap = Number.parseInt(localStorage.getItem(LAST_RECAP_KEY) ?? "0", 10);
        if (!Number.isFinite(lastRecap) || Date.now() - lastRecap >= recapIntervalMs) {
          const recap = await activity.dailyBrief(24 * 7);
          if (recap.totalEntries > 0) {
            localStorage.setItem(LAST_RECAP_KEY, String(Date.now()));
            toast.message("This week with Acorn", {
              description: recap.summary,
              duration: 12000,
            });
          }
        }
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
      const target = event.payload;
      if (
        target === "settings" ||
        target === "chat" ||
        target === "calendar" ||
        target === "canvas" ||
        target === "input" ||
        target === "stash"
      ) {
        setView(target);
      }
    });

    // Re-send the latest reply summary when the summary overlay (re)mounts, in
    // case its listener registered just after our initial emit.
    const unlistenSummaryRequest = listen("summary:request", () => {
      if (latestSummary.current) void emit("summary:show", latestSummary.current);
    });

    const unlistenPinShortcut = listen("shortcut:pin-response", async () => {
      const conversation = useChatStore.getState().current;
      const lastReply = conversation?.messages
        .filter((m) => m.role === "assistant" && m.content.trim().length > 0)
        .at(-1);
      if (!lastReply) {
        toast.message("Nothing to pin yet", {
          description: "Send a message first, then pin the reply.",
          id: "shortcut-pin-empty",
        });
        return;
      }
      try {
        const pin = await pins.create({
          conversationId: conversation?.id ?? null,
          messageId: lastReply.id,
          label: conversation?.title?.trim() || "Pinned",
          content: lastReply.content,
        });
        await pins.openWindow(pin.id, pin.x, pin.y);
        void navigator.clipboard.writeText(lastReply.content).catch(() => {});
        const preview =
          lastReply.content.length > 80
            ? `${lastReply.content.slice(0, 80).trim()}…`
            : lastReply.content.trim();
        toast.success("Pinned to desktop", {
          description: preview,
          id: "shortcut-pin",
        });
      } catch (err) {
        toast.error("Could not pin", {
          description: err instanceof Error ? err.message : String(err),
          id: "shortcut-pin-error",
        });
      }
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
      try {
        await invoke("toggle_quick");
      } catch (err) {
        console.error("quick window toggle failed", err);
      }
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
      void unlistenSummaryRequest.then((fn) => fn());
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
      lastReply.content.length > 240
        ? `${lastReply.content.slice(0, 240).trim()}…`
        : lastReply.content.trim();
    const payload: SummaryPayload = {
      content: lastReply.content,
      preview,
      providerId: chatCurrent?.providerId ?? null,
      ts: Date.now(),
    };
    latestSummary.current = payload;
    void overlay
      .showSummary()
      .then(() => emit("summary:show", payload))
      .catch(() => {});
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
