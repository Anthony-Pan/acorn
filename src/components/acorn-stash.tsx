import { AnimatePresence } from "framer-motion";
import {
  CalendarDays,
  FileText,
  MessageCircle,
  Plus,
  Settings as SettingsIcon,
  Sparkles,
} from "lucide-react";

import { AcornCard } from "@/components/acorn-card";
import { AcornLogo } from "@/components/acorn-logo";
import { Button } from "@/components/ui/button";
import { strings } from "@/lib/i18n";
import { useSessionStore } from "@/stores/session";
import { useSettingsStore } from "@/stores/settings";
import type { Task, TaskStatus } from "@/types/db";

interface AcornStashProps {
  onOpenSettings: () => void;
  onOpenChat: () => void;
  onOpenCalendar: () => void;
  onOpenCanvas: () => void;
  onAddMore: () => void;
}

export function AcornStash({
  onOpenSettings,
  onOpenChat,
  onOpenCalendar,
  onOpenCanvas,
  onAddMore,
}: AcornStashProps) {
  const current = useSessionStore((s) => s.current);
  const updateTaskStatus = useSessionStore((s) => s.updateTaskStatus);
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  if (!current) return null;
  const tasks = [...current.tasks].sort((a, b) => a.orderIndex - b.orderIndex);
  const counts = tally(tasks);
  const activeTaskId = tasks.find((t) => t.status === "in_progress")?.id ?? null;

  const handleStart = async (task: Task) => {
    if (activeTaskId && activeTaskId !== task.id) {
      await updateTaskStatus(activeTaskId, "pending");
    }
    await updateTaskStatus(task.id, "in_progress");
  };
  const handleDone = (task: Task) => updateTaskStatus(task.id, "completed");
  const handleSkip = (task: Task) => updateTaskStatus(task.id, "skipped");

  return (
    <div className="min-h-screen bg-background">
      <header
        data-tauri-drag-region
        className="flex h-[52px] items-center justify-between pl-[76px] pr-4 select-none"
      >
        <div className="pointer-events-none flex items-center gap-2.5">
          <AcornLogo size={24} />
          <div className="text-[13px] font-semibold text-foreground">{t.todaysStash}</div>
        </div>
        <div className="flex items-center gap-3">
          <CountsLine counts={counts} t={t} />
          <Button variant="outline" size="sm" onClick={onOpenChat}>
            <MessageCircle />
            {t.chat}
          </Button>
          <Button variant="outline" size="icon-sm" onClick={onOpenCanvas} aria-label="Canvas">
            <FileText className="w-3.5 h-3.5" />
          </Button>
          <Button variant="outline" size="icon-sm" onClick={onOpenCalendar} aria-label="Calendar">
            <CalendarDays />
          </Button>
          <Button variant="outline" size="icon-sm" onClick={onOpenSettings} aria-label="Settings">
            <SettingsIcon />
          </Button>
        </div>
      </header>

      <div className="max-w-2xl mx-auto px-7 pb-7 pt-4">
        {current.aiSummary ? (
          <div className="bg-acorn-orange/5 border-l-2 border-acorn-orange rounded-r-md px-3.5 py-2.5 mb-5">
            <div className="text-[13px] text-muted-foreground leading-relaxed flex gap-2">
              <Sparkles className="w-3.5 h-3.5 text-acorn-orange flex-shrink-0 mt-0.5" />
              <span>{current.aiSummary}</span>
            </div>
          </div>
        ) : null}

        <div className="space-y-2.5">
          <AnimatePresence initial={false}>
            {tasks.map((task) => (
              <AcornCard
                key={task.id}
                task={task}
                onStart={handleStart}
                onDone={handleDone}
                onSkip={handleSkip}
              />
            ))}
          </AnimatePresence>
        </div>

        <button
          type="button"
          onClick={onAddMore}
          className="mt-3.5 w-full inline-flex items-center justify-center gap-1.5 py-2.5 text-[13px] text-muted-foreground rounded-lg border border-dashed border-border hover:bg-muted/50 transition-colors"
        >
          <Plus className="w-3.5 h-3.5" />
          {t.addAnother}
        </button>
      </div>
    </div>
  );
}

function CountsLine({
  counts,
  t,
}: {
  counts: ReturnType<typeof tally>;
  t: ReturnType<typeof strings>;
}) {
  return (
    <div className="text-[11px] text-muted-foreground tabular-nums">
      <span className="font-medium text-acorn-brown-deep">{t.doneCount(counts.done)}</span>
      <span className="mx-1.5">·</span>
      <span className="font-medium text-acorn-orange">{t.inProgressCount(counts.inProgress)}</span>
      <span className="mx-1.5">·</span>
      {t.toGoCount(counts.remaining)}
    </div>
  );
}

function tally(tasks: Task[]): Record<"done" | "inProgress" | "remaining", number> {
  const counts: Record<TaskStatus, number> = {
    pending: 0,
    in_progress: 0,
    completed: 0,
    skipped: 0,
  };
  for (const task of tasks) counts[task.status]++;
  return {
    done: counts.completed,
    inProgress: counts.in_progress,
    remaining: counts.pending + counts.skipped,
  };
}
