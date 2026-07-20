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
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <header className="mb-6">
        <h1 className="text-[20px] font-semibold text-foreground">{t.displayPanelTitle}</h1>
        <p className="mt-1 text-[13px] text-muted-foreground">{t.displayPanelSubtitle}</p>
      </header>

      <div className="max-w-xl divide-y-[0.5px] divide-border rounded-lg border-[0.5px] border-border bg-card shadow-[var(--shadow-card)]">
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
    <div className="flex items-center justify-between gap-6 px-4 py-3">
      <div className="min-w-0 flex-1">
        <div className="text-[13px] text-foreground">{label}</div>
        <div className="mt-0.5 text-[11px] text-muted-foreground">{caption}</div>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={onChange}
        className={cn(
          "relative h-6 w-10 flex-shrink-0 rounded-full transition-colors",
          checked ? "bg-acorn-orange" : "bg-muted-foreground/25",
        )}
      >
        <span
          className={cn(
            "absolute top-0.5 h-5 w-5 rounded-full bg-white shadow-sm transition-transform",
            checked ? "translate-x-[18px]" : "translate-x-0.5",
          )}
        />
      </button>
    </div>
  );
}
