import { useEffect, useState } from "react";

import { settings } from "@/lib/db";
import { strings } from "@/lib/i18n";
import { pet } from "@/lib/pet";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/stores/settings";

const PET_HIDDEN_KEY = "pet-hidden";
const SOUNDS_MUTED_KEY = "acorn:sounds-muted";

function readBool(key: string): boolean {
  try {
    return localStorage.getItem(key) === "true";
  } catch {
    return false;
  }
}

function writeBool(key: string, value: boolean): void {
  try {
    if (value) localStorage.setItem(key, "true");
    else localStorage.removeItem(key);
  } catch {}
}

export function DisplayPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const [cornieHidden, setCornieHidden] = useState(false);
  const [soundsMuted, setSoundsMuted] = useState(false);

  useEffect(() => {
    void settings.get(PET_HIDDEN_KEY).then((value) => setCornieHidden(value === "true"));
    setSoundsMuted(readBool(SOUNDS_MUTED_KEY));
  }, []);

  const toggleCornie = () => {
    const nextHidden = !cornieHidden;
    setCornieHidden(nextHidden);
    void settings.set(PET_HIDDEN_KEY, nextHidden ? "true" : "");
    if (nextHidden) void pet.hide();
    else void pet.show();
  };

  const toggleSounds = () => {
    const next = !soundsMuted;
    writeBool(SOUNDS_MUTED_KEY, next);
    setSoundsMuted(next);
  };

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-6">
        <h1 className="text-lg font-medium text-foreground">{t.displayPanelTitle}</h1>
        <p className="text-xs text-muted-foreground mt-1">{t.displayPanelSubtitle}</p>
      </header>

      <div className="space-y-4 max-w-2xl">
        <ToggleRow
          label={t.displayCornieLabel}
          caption={t.displayCornieCaption}
          checked={!cornieHidden}
          onChange={toggleCornie}
        />
        <ToggleRow
          label={t.displaySoundsLabel}
          caption={t.displaySoundsCaption}
          checked={!soundsMuted}
          onChange={toggleSounds}
        />
      </div>
    </section>
  );
}

interface ToggleRowProps {
  label: string;
  caption: string;
  checked: boolean;
  onChange: () => void;
}

function ToggleRow({ label, caption, checked, onChange }: ToggleRowProps) {
  return (
    <div className="flex items-start justify-between gap-6 border-[0.5px] border-border rounded-md px-4 py-3 bg-card">
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-foreground">{label}</div>
        <div className="text-xs text-muted-foreground mt-0.5">{caption}</div>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={onChange}
        className={cn(
          "relative w-10 h-6 rounded-full transition-colors flex-shrink-0",
          checked ? "bg-acorn-orange" : "bg-muted",
        )}
      >
        <span
          className={cn(
            "absolute top-0.5 w-5 h-5 rounded-full bg-acorn-paper shadow transition-transform",
            checked ? "translate-x-[18px]" : "translate-x-0.5",
          )}
        />
      </button>
    </div>
  );
}
