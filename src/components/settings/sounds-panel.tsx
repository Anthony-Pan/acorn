import { Loader2, Play, Volume2, VolumeX } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { strings } from "@/lib/i18n";
import { isMuted, setMuted, sounds } from "@/lib/sounds";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/stores/settings";

interface SoundEntry {
  key: keyof typeof sounds;
  triggerLabelEn: string;
  triggerLabelZh: string;
  descriptionEn: string;
  descriptionZh: string;
}

const ENTRIES: SoundEntry[] = [
  {
    key: "chime",
    triggerLabelEn: "Chime",
    triggerLabelZh: "回复完成",
    descriptionEn: "Two-note ascending chime when a chat reply lands successfully.",
    descriptionZh: "AI 回复落地时,两个音的轻轻一叮。",
  },
  {
    key: "error",
    triggerLabelEn: "Error",
    triggerLabelZh: "出错",
    descriptionEn: "Single low tone when a chat or stash fails.",
    descriptionZh: "对话或 stash 出错时,一个低沉的提示音。",
  },
  {
    key: "pop",
    triggerLabelEn: "Pop",
    triggerLabelZh: "弹出",
    descriptionEn: "Tiny pop reserved for upcoming micro-interactions.",
    descriptionZh: "未来微交互预留的小弹音。",
  },
];

export function SoundsPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);
  const [muted, setMutedState] = useState(false);
  const [playing, setPlaying] = useState<string | null>(null);

  useEffect(() => {
    setMutedState(isMuted());
  }, []);

  const toggleMute = () => {
    const next = !muted;
    setMuted(next);
    setMutedState(next);
  };

  const preview = (entry: SoundEntry) => {
    setPlaying(entry.key);
    sounds[entry.key]();
    window.setTimeout(() => setPlaying(null), 400);
  };

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-5">
        <h1 className="text-lg font-medium text-foreground">{t.soundsPanelTitle}</h1>
        <p className="text-xs text-muted-foreground mt-1">{t.soundsPanelSubtitle}</p>
      </header>

      <div className="max-w-2xl">
        <button
          type="button"
          onClick={toggleMute}
          className={cn(
            "w-full mb-4 flex items-center justify-between gap-3 border-[0.5px] rounded-md px-4 py-3 transition-colors",
            muted ? "bg-muted/40 border-border" : "bg-acorn-orange/5 border-acorn-orange/30",
          )}
        >
          <div className="flex items-center gap-2">
            {muted ? (
              <VolumeX className="w-4 h-4 text-muted-foreground" />
            ) : (
              <Volume2 className="w-4 h-4 text-acorn-orange" />
            )}
            <span className="text-sm font-medium text-foreground">
              {muted ? t.soundsMutedLabel : t.soundsOnLabel}
            </span>
          </div>
          <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
            {muted ? t.soundsClickToUnmute : t.soundsClickToMute}
          </span>
        </button>

        <div className="space-y-2">
          {ENTRIES.map((entry) => (
            <article
              key={entry.key}
              className="border-[0.5px] border-border rounded-md px-4 py-3 bg-card flex items-start gap-3"
            >
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium text-foreground">
                  {language.startsWith("zh") ? entry.triggerLabelZh : entry.triggerLabelEn}
                </div>
                <div className="text-xs text-muted-foreground mt-0.5">
                  {language.startsWith("zh") ? entry.descriptionZh : entry.descriptionEn}
                </div>
              </div>
              <Button variant="outline" size="sm" onClick={() => preview(entry)} disabled={muted}>
                {playing === entry.key ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <Play className="w-3 h-3" />
                )}
                {t.soundsPreviewButton}
              </Button>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
