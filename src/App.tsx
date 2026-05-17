import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";

import { AcornStash } from "@/components/acorn-stash";
import { SettingsPage } from "@/components/settings/settings-page";
import { TaskInput } from "@/components/task-input";
import { useProvidersStore } from "@/stores/providers";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";

type View = "input" | "stash" | "settings";

function App() {
  const hydrateSession = useSessionStore((s) => s.hydrate);
  const hydrateSettings = useSettingsStore((s) => s.hydrate);
  const hydrateProviders = useProvidersStore((s) => s.hydrate);
  const current = useSessionStore((s) => s.current);
  const isStashing = useSessionStore((s) => s.isStashing);

  const [view, setView] = useState<View>("input");

  useEffect(() => {
    void Promise.all([hydrateSettings(), hydrateProviders(), hydrateSession()]);
  }, [hydrateSession, hydrateSettings, hydrateProviders]);

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

    return () => {
      void unlistenSubmit.then((fn) => fn());
      void unlistenNavigate.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (view === "settings") return;
    if (current && current.tasks.length > 0) {
      setView("stash");
    } else if (!isStashing) {
      setView("input");
    }
  }, [current, isStashing, view]);

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={view}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.18 }}
      >
        {view === "settings" ? (
          <SettingsPage
            onClose={() => setView(current && current.tasks.length > 0 ? "stash" : "input")}
          />
        ) : view === "stash" ? (
          <AcornStash
            onOpenSettings={() => setView("settings")}
            onAddMore={() => {
              useSessionStore.getState().startNew();
              setView("input");
            }}
          />
        ) : (
          <TaskInput onOpenSettings={() => setView("settings")} />
        )}
      </motion.div>
    </AnimatePresence>
  );
}

export default App;
