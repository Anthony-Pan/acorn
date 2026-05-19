import { motion } from "framer-motion";
import { Check, Clock } from "lucide-react";
import { useEffect, useState } from "react";

import { PriorityBadge } from "@/components/priority-badge";
import { Button } from "@/components/ui/button";
import {
  formatClock,
  formatDurationMinutes,
  formatTimerSince,
  formatTookSince,
} from "@/lib/format";
import { cn } from "@/lib/utils";
import type { Task } from "@/types/db";

interface AcornCardProps {
  task: Task;
  onStart: (task: Task) => void;
  onDone: (task: Task) => void;
  onSkip: (task: Task) => void;
}

export function AcornCard({ task, onStart, onDone, onSkip }: AcornCardProps) {
  if (task.status === "completed") return <CompletedCard task={task} />;
  if (task.status === "skipped") return <SkippedCard task={task} />;
  if (task.status === "in_progress")
    return <InProgressCard task={task} onDone={onDone} onSkip={onSkip} />;
  return <PendingCard task={task} onStart={onStart} />;
}

function PendingCard({ task, onStart }: { task: Task; onStart: (task: Task) => void }) {
  return (
    <motion.button
      layout
      initial={{ opacity: 0, y: 24, scale: 0.86 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, scale: 0.94 }}
      transition={{ type: "spring", stiffness: 320, damping: 22 }}
      type="button"
      onClick={() => onStart(task)}
      className={cn(
        "w-full text-left bg-card border-[0.5px] border-border rounded-xl px-4 py-3.5",
        "hover:border-acorn-orange/40 transition-colors group",
      )}
    >
      <div className="flex items-center gap-2.5">
        <div className="w-[22px] h-[22px] rounded-full border-[1.5px] border-border group-hover:border-acorn-orange/60 transition-colors" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between gap-2">
            <div className="text-sm font-medium text-foreground truncate">{task.title}</div>
            <PriorityBadge priority={task.priority} />
          </div>
          <div className="text-[11px] text-muted-foreground mt-0.5">
            Est. {formatDurationMinutes(task.durationMinutes)}
            {task.description ? ` · ${task.description}` : null}
          </div>
        </div>
      </div>
    </motion.button>
  );
}

function InProgressCard({
  task,
  onDone,
  onSkip,
}: {
  task: Task;
  onDone: (task: Task) => void;
  onSkip: (task: Task) => void;
}) {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.18 }}
      className={cn(
        "bg-card rounded-xl px-4 py-4 border-2 border-acorn-orange",
        "shadow-[0_0_0_4px_rgba(181,128,63,0.1)]",
      )}
    >
      <div className="flex items-start gap-3">
        <div className="w-[26px] h-[26px] rounded-full border-2 border-acorn-orange flex items-center justify-center flex-shrink-0 mt-0.5">
          <div className="w-2 h-2 rounded-full bg-acorn-orange" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between gap-2 mb-1">
            <div className="text-[15px] font-medium text-foreground">{task.title}</div>
            <PriorityBadge priority={task.priority} />
          </div>
          {task.description ? (
            <div className="text-xs text-muted-foreground mb-3 leading-relaxed">
              Est. {formatDurationMinutes(task.durationMinutes)} · {task.description}
            </div>
          ) : (
            <div className="text-xs text-muted-foreground mb-3">
              Est. {formatDurationMinutes(task.durationMinutes)}
            </div>
          )}
          <div className="flex items-center gap-2 pt-2.5 border-t-[0.5px] border-acorn-brown/10">
            {task.startedAt ? <ProgressTimer startedAt={task.startedAt} /> : <span />}
            <div className="ml-auto flex gap-2">
              <Button variant="outline" size="sm" onClick={() => onSkip(task)}>
                Skip
              </Button>
              <Button
                size="sm"
                onClick={() => onDone(task)}
                className="bg-acorn-olive text-acorn-paper hover:bg-acorn-olive/85"
              >
                Done
              </Button>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function CompletedCard({ task }: { task: Task }) {
  const took =
    task.startedAt && task.completedAt ? formatTookSince(task.startedAt, task.completedAt) : null;
  const finishedAt = task.completedAt ? formatClock(task.completedAt) : null;

  return (
    <motion.div
      layout
      initial={{ opacity: 0 }}
      animate={{ opacity: 0.55 }}
      transition={{ duration: 0.2 }}
      className="bg-card border-[0.5px] border-border rounded-xl px-4 py-3.5"
    >
      <div className="flex items-center gap-2.5">
        <div className="w-[22px] h-[22px] rounded-full bg-acorn-olive flex items-center justify-center">
          <Check className="w-3.5 h-3.5 text-acorn-paper" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium text-foreground line-through">{task.title}</div>
          <div className="text-[11px] text-muted-foreground mt-0.5">
            {finishedAt ? `Done at ${finishedAt}` : "Done"}
            {took ? ` · took ${took}` : null}
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function SkippedCard({ task }: { task: Task }) {
  return (
    <motion.div
      layout
      initial={{ opacity: 0 }}
      animate={{ opacity: 0.4 }}
      transition={{ duration: 0.2 }}
      className="bg-card border-[0.5px] border-dashed border-border rounded-xl px-4 py-3.5"
    >
      <div className="flex items-center gap-2.5">
        <div className="w-[22px] h-[22px] rounded-full border-[1.5px] border-dashed border-border" />
        <div className="flex-1 min-w-0">
          <div className="text-sm text-muted-foreground line-through">{task.title}</div>
          <div className="text-[11px] text-muted-foreground mt-0.5">Skipped</div>
        </div>
      </div>
    </motion.div>
  );
}

function ProgressTimer({ startedAt }: { startedAt: string }) {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, []);

  return (
    <div className="flex items-center gap-1.5 text-acorn-orange text-xs font-medium">
      <Clock className="w-3.5 h-3.5" />
      {formatTimerSince(startedAt, now)} in progress
    </div>
  );
}
