import { Pin, SkipForward } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { PriorityBadge } from "@/components/priority-badge";
import { Button } from "@/components/ui/button";
import type { Priority } from "@/types/db";

interface InlineTask {
  title: string;
  description: string | null;
  durationMinutes: number;
  priority: Priority;
}

interface TaskActionCardsProps {
  json: string;
}

const FALLBACK_DURATION = 15;

function normalizePriority(value: unknown): Priority {
  if (value === "high" || value === "low") return value;
  return "medium";
}

function parseInlineTasks(raw: string): InlineTask[] | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    const list = Array.isArray(parsed) ? parsed : null;
    if (!list) return null;
    return list
      .filter((item): item is Record<string, unknown> => typeof item === "object" && item !== null)
      .map((item) => {
        const title = typeof item.title === "string" ? item.title.trim() : "";
        if (!title) return null;
        const durationValue = item.durationMinutes ?? item.minutes ?? item.duration;
        const durationMinutes =
          typeof durationValue === "number" && Number.isFinite(durationValue) && durationValue > 0
            ? Math.min(Math.round(durationValue), 8 * 60)
            : FALLBACK_DURATION;
        const description = typeof item.description === "string" ? item.description : null;
        return {
          title,
          description,
          durationMinutes,
          priority: normalizePriority(item.priority),
        };
      })
      .filter((task): task is InlineTask => task !== null);
  } catch {
    return null;
  }
}

export function TaskActionCards({ json }: TaskActionCardsProps) {
  const tasks = useMemo(() => parseInlineTasks(json), [json]);
  const [skipped, setSkipped] = useState<Set<number>>(() => new Set());

  if (!tasks || tasks.length === 0) {
    return (
      <pre className="text-xs font-mono bg-muted/40 rounded-md p-3 overflow-x-auto">
        <code>{json}</code>
      </pre>
    );
  }

  const visible = tasks.map((task, idx) => ({ task, idx })).filter(({ idx }) => !skipped.has(idx));

  if (visible.length === 0) {
    return <div className="text-xs text-muted-foreground italic">All suggested tasks skipped.</div>;
  }

  return (
    <div className="not-prose grid gap-2 my-3">
      {visible.map(({ task, idx }) => (
        <article
          key={idx}
          className="border-[0.5px] border-border rounded-lg px-3 py-2.5 bg-card flex items-start gap-3"
        >
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <PriorityBadge priority={task.priority} />
              <span className="text-[10px] text-muted-foreground">{task.durationMinutes}m</span>
            </div>
            <div className="text-sm font-medium text-foreground">{task.title}</div>
            {task.description ? (
              <div className="text-xs text-muted-foreground mt-0.5">{task.description}</div>
            ) : null}
          </div>
          <div className="flex flex-col gap-1 flex-shrink-0">
            <Button
              variant="outline"
              size="icon-sm"
              aria-label="Pin to desktop"
              onClick={() =>
                toast.message("Pin to desktop", {
                  description: "Pinning task cards lands with the desktop overlay phase.",
                })
              }
            >
              <Pin className="w-3.5 h-3.5" />
            </Button>
            <Button
              variant="outline"
              size="icon-sm"
              aria-label="Skip this suggestion"
              onClick={() =>
                setSkipped((prev) => {
                  const next = new Set(prev);
                  next.add(idx);
                  return next;
                })
              }
            >
              <SkipForward className="w-3.5 h-3.5" />
            </Button>
          </div>
        </article>
      ))}
    </div>
  );
}
