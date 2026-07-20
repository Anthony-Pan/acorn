import { Loader2, Play, RotateCcw, Upload, Volume2, VolumeX } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { strings } from "@/lib/i18n";
import {
  clearCustomSound,
  getCustomSound,
  isMuted,
  type SoundKey,
  setCustomSound,
  setMuted,
  sounds,
} from "@/lib/sounds";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/stores/settings";

interface SoundEntry {
  key: SoundKey;
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

const MAX_SOUND_BYTES = 1_000_000;

export function SoundsPanel() {
  const language = useSettingsStore((s) => s.language);
  const zh = language.startsWith("zh");
  const t = strings(language);
  const [muted, setMutedState] = useState(false);
  const [playing, setPlaying] = useState<string | null>(null);
  const [custom, setCustom] = useState<Record<string, boolean>>({});
  const [tooLarge, setTooLarge] = useState<Record<string, boolean>>({});

  useEffect(() => {
    setMutedState(isMuted());
    const present: Record<string, boolean> = {};
    for (const entry of ENTRIES) present[entry.key] = getCustomSound(entry.key) !== null;
    setCustom(present);
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

  const handleUpload = (key: SoundKey, file: File | undefined) => {
    if (!file) return;
    if (file.size > MAX_SOUND_BYTES) {
      setTooLarge((prev) => ({ ...prev, [key]: true }));
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result !== "string") return;
      setCustomSound(key, reader.result);
      setCustom((prev) => ({ ...prev, [key]: true }));
      setTooLarge((prev) => ({ ...prev, [key]: false }));
    };
    reader.readAsDataURL(file);
  };

  const handleReset = (key: SoundKey) => {
    clearCustomSound(key);
    setCustom((prev) => ({ ...prev, [key]: false }));
    setTooLarge((prev) => ({ ...prev, [key]: false }));
  };

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <header className="mb-5">
        <h1 className="text-[20px] font-semibold text-foreground">{t.soundsPanelTitle}</h1>
        <p className="mt-1 text-[13px] text-muted-foreground">{t.soundsPanelSubtitle}</p>
      </header>

      <div className="max-w-2xl">
        <button
          type="button"
          onClick={toggleMute}
          className={cn(
            "mb-4 flex w-full items-center justify-between gap-3 rounded-lg border-[0.5px] px-4 py-3 shadow-[var(--shadow-card)] transition-colors",
            muted ? "border-border bg-muted/40" : "border-acorn-orange/30 bg-acorn-orange/5",
          )}
        >
          <div className="flex items-center gap-2">
            {muted ? (
              <VolumeX className="w-4 h-4 text-muted-foreground" />
            ) : (
              <Volume2 className="w-4 h-4 text-acorn-orange" />
            )}
            <span className="text-[13px] font-medium text-foreground">
              {muted ? t.soundsMutedLabel : t.soundsOnLabel}
            </span>
          </div>
          <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
            {muted ? t.soundsClickToUnmute : t.soundsClickToMute}
          </span>
        </button>

        <div className="divide-y-[0.5px] divide-border overflow-hidden rounded-lg border-[0.5px] border-border bg-card shadow-[var(--shadow-card)]">
          {ENTRIES.map((entry) => (
            <article key={entry.key} className="px-4 py-3">
              <div className="flex items-start gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-[13px] text-foreground">
                      {zh ? entry.triggerLabelZh : entry.triggerLabelEn}
                    </span>
                    {custom[entry.key] ? (
                      <span className="rounded-full bg-acorn-orange/15 px-1.5 py-0.5 text-[9px] font-medium uppercase tracking-wider text-acorn-brown">
                        {zh ? "自定义" : "Custom"}
                      </span>
                    ) : null}
                  </div>
                  <div className="mt-0.5 text-[11px] text-muted-foreground">
                    {zh ? entry.descriptionZh : entry.descriptionEn}
                  </div>
                  {tooLarge[entry.key] ? (
                    <div className="mt-1 text-[11px] text-acorn-red">
                      {zh ? "文件太大(上限 1MB)" : "File too large (max 1MB)"}
                    </div>
                  ) : null}
                </div>
                <div className="flex shrink-0 items-center gap-1.5">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => preview(entry)}
                    disabled={muted}
                  >
                    {playing === entry.key ? (
                      <Loader2 className="w-3 h-3 animate-spin" />
                    ) : (
                      <Play className="w-3 h-3" />
                    )}
                    {t.soundsPreviewButton}
                  </Button>
                  <label
                    className="inline-flex h-8 cursor-pointer items-center gap-1 rounded-md border-[0.5px] border-border px-2.5 text-[13px] font-medium text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
                    title={zh ? "上传自定义音频" : "Upload custom audio"}
                  >
                    <Upload className="w-3 h-3" />
                    {zh ? "上传" : "Upload"}
                    <input
                      type="file"
                      accept="audio/*"
                      className="hidden"
                      onChange={(e) => {
                        handleUpload(entry.key, e.target.files?.[0]);
                        e.target.value = "";
                      }}
                    />
                  </label>
                  {custom[entry.key] ? (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleReset(entry.key)}
                      title={zh ? "恢复默认音效" : "Reset to default"}
                    >
                      <RotateCcw className="w-3 h-3" />
                    </Button>
                  ) : null}
                </div>
              </div>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
