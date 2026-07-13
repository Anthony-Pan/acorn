import { AnimatePresence, motion } from "framer-motion";
import {
  ArrowLeft,
  Check,
  ChevronLeft,
  ChevronRight,
  CircleDashed,
  Clock,
  Sparkles,
  SunMedium,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { PriorityBadge } from "@/components/priority-badge";
import { Button } from "@/components/ui/button";
import {
  buildMonthGrid,
  dateKey,
  formatDayLabel,
  formatMonth,
  groupSessionsByDate,
  isSameDay,
  isSameMonth,
  isToday,
  nextMonth,
  previousMonth,
  tallyTasks,
  todayKey,
} from "@/lib/calendar";
import { sessions } from "@/lib/db";
import { formatClock, formatDurationMinutes } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { Session, SessionWithTasks, Task } from "@/types/db";

const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;
const RECENT_SESSIONS_LIMIT = 365;

interface CalendarViewProps {
  onBack: () => void;
}

export function CalendarView({ onBack }: CalendarViewProps) {
  const [cursor, setCursor] = useState<Date>(() => new Date());
  const [selected, setSelected] = useState<Date>(() => new Date());
  const [allSessions, setAllSessions] = useState<Session[]>([]);
  const [dayDetail, setDayDetail] = useState<SessionWithTasks | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    sessions
      .listRecent(RECENT_SESSIONS_LIMIT)
      .then((rows) => {
        if (!cancelled) setAllSessions(rows);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          setError(message);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const sessionsByDay = useMemo(() => groupSessionsByDate(allSessions), [allSessions]);
  const grid = useMemo(() => buildMonthGrid(cursor), [cursor]);

  useEffect(() => {
    const key = dateKey(selected);
    const sessionsForDay = sessionsByDay.get(key);
    const firstSession = sessionsForDay?.[0];
    if (!firstSession) {
      setDayDetail(null);
      return;
    }
    let cancelled = false;
    setLoadingDetail(true);
    sessions
      .get(firstSession.id)
      .then((detail) => {
        if (!cancelled) setDayDetail(detail);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          setError(message);
          setDayDetail(null);
        }
      })
      .finally(() => {
        if (!cancelled) setLoadingDetail(false);
      });
    return () => {
      cancelled = true;
    };
  }, [selected, sessionsByDay]);

  const selectedKey = dateKey(selected);
  const hasStashForSelected = sessionsByDay.has(selectedKey);
  const selectedIsToday = selectedKey === todayKey();

  return (
    <div className="min-h-screen bg-background">
      <header
        data-tauri-drag-region
        className="flex h-[52px] items-center justify-between pl-[76px] pr-4 select-none"
      >
        <div className="pointer-events-none flex items-center gap-2.5">
          <AcornLogo size={24} />
          <div className="text-[13px] font-semibold text-foreground leading-none">Calendar</div>
        </div>
        <Button variant="outline" size="sm" onClick={onBack}>
          <ArrowLeft />
          Back
        </Button>
      </header>

      <div className="max-w-2xl mx-auto px-7 pt-3 pb-7">
        <div className="bg-card border-[0.5px] border-border rounded-lg shadow-[var(--shadow-card)] p-5 mb-5">
          <div className="flex items-center justify-between mb-4">
            <button
              type="button"
              onClick={() => setCursor(previousMonth(cursor))}
              className="w-8 h-8 inline-flex items-center justify-center rounded-md text-muted-foreground hover:bg-muted/60 hover:text-foreground transition-colors"
              aria-label="Previous month"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>
            <div className="flex items-center gap-3">
              <div className="text-[15px] font-semibold text-foreground select-none">
                {formatMonth(cursor)}
              </div>
              {!isSameMonth(cursor, new Date()) ? (
                <button
                  type="button"
                  onClick={() => {
                    const now = new Date();
                    setCursor(now);
                    setSelected(now);
                  }}
                  className="text-[11px] font-medium text-primary hover:text-primary/80 transition-colors inline-flex items-center gap-1"
                >
                  <SunMedium className="w-3 h-3" />
                  Today
                </button>
              ) : null}
            </div>
            <button
              type="button"
              onClick={() => setCursor(nextMonth(cursor))}
              className="w-8 h-8 inline-flex items-center justify-center rounded-md text-muted-foreground hover:bg-muted/60 hover:text-foreground transition-colors"
              aria-label="Next month"
            >
              <ChevronRight className="w-4 h-4" />
            </button>
          </div>

          <div className="grid grid-cols-7 gap-1 mb-1.5">
            {WEEKDAYS.map((label) => (
              <div
                key={label}
                className="text-[11px] font-medium text-muted-foreground text-center py-1 select-none"
              >
                {label}
              </div>
            ))}
          </div>

          <div className="grid grid-cols-7 gap-1">
            {grid.map((day) => (
              <DayCell
                key={day.toISOString()}
                day={day}
                cursor={cursor}
                selected={selected}
                hasStash={sessionsByDay.has(dateKey(day))}
                onSelect={setSelected}
              />
            ))}
          </div>
        </div>

        {error ? (
          <div className="bg-destructive/5 border-[0.5px] border-destructive/30 rounded-md px-3.5 py-2.5 mb-4 text-[11px] text-destructive">
            {error}
          </div>
        ) : null}

        <AnimatePresence mode="wait">
          <motion.div
            key={selectedKey}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.18, ease: [0.25, 0.1, 0.25, 1] }}
          >
            <DayPanel
              date={selected}
              detail={dayDetail}
              hasStash={hasStashForSelected}
              loading={loadingDetail}
              isToday={selectedIsToday}
            />
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  );
}

interface DayCellProps {
  day: Date;
  cursor: Date;
  selected: Date;
  hasStash: boolean;
  onSelect: (day: Date) => void;
}

function DayCell({ day, cursor, selected, hasStash, onSelect }: DayCellProps) {
  const inMonth = isSameMonth(day, cursor);
  const isSelected = isSameDay(day, selected);
  const today = isToday(day);

  return (
    <button
      type="button"
      onClick={() => onSelect(day)}
      className={cn(
        "relative aspect-square rounded-full text-[13px] tabular-nums flex flex-col items-center justify-center transition-colors duration-150",
        inMonth ? "text-foreground" : "text-muted-foreground/40",
        today
          ? "bg-primary text-primary-foreground font-semibold"
          : isSelected
            ? "bg-accent text-accent-foreground font-medium"
            : "hover:bg-muted/60",
        today && isSelected ? "ring-2 ring-ring/30" : "",
      )}
    >
      <span className={cn("leading-none", isSelected ? "font-semibold" : "")}>{day.getDate()}</span>
      {hasStash ? (
        <span
          className={cn(
            "absolute bottom-1.5 w-1 h-1 rounded-full",
            today ? "bg-primary-foreground" : "bg-primary",
          )}
        />
      ) : null}
    </button>
  );
}

interface DayPanelProps {
  date: Date;
  detail: SessionWithTasks | null;
  hasStash: boolean;
  loading: boolean;
  isToday: boolean;
}

function DayPanel({ date, detail, hasStash, loading, isToday }: DayPanelProps) {
  const tasks = useMemo(() => {
    if (!detail) return [];
    return [...detail.tasks].sort((a, b) => a.orderIndex - b.orderIndex);
  }, [detail]);
  const stats = useMemo(() => tallyTasks(tasks), [tasks]);

  return (
    <div className="bg-card border-[0.5px] border-border rounded-lg shadow-[var(--shadow-card)] p-5">
      <div className="flex items-baseline justify-between mb-3">
        <div className="text-[13px] font-semibold text-foreground">
          {formatDayLabel(date)}
          {isToday ? (
            <span className="ml-2 text-[11px] font-medium uppercase tracking-wider text-primary">
              Today
            </span>
          ) : null}
        </div>
        {tasks.length > 0 ? (
          <div className="text-[11px] text-muted-foreground tabular-nums">
            <span className="font-medium text-foreground">{stats.completed}</span> done ·{" "}
            <span className="font-medium text-primary">{stats.inProgress}</span> in progress ·{" "}
            {stats.pending + stats.skipped} to go
          </div>
        ) : null}
      </div>

      {detail?.aiSummary ? (
        <div className="bg-primary/5 border-l-2 border-primary/60 rounded-r-md px-3.5 py-2.5 mb-4">
          <div className="text-[13px] text-muted-foreground leading-relaxed flex gap-2">
            <Sparkles className="w-3.5 h-3.5 text-primary flex-shrink-0 mt-0.5" />
            <span>{detail.aiSummary}</span>
          </div>
        </div>
      ) : null}

      {loading ? (
        <EmptyState label="Loading…" icon={<CircleDashed className="w-4 h-4 animate-spin" />} />
      ) : !hasStash ? (
        <EmptyState
          label={isToday ? "No stash for today yet" : "Nothing was stashed on this day"}
          icon={<CircleDashed className="w-4 h-4" />}
        />
      ) : tasks.length === 0 ? (
        <EmptyState
          label="Session exists but has no tasks"
          icon={<CircleDashed className="w-4 h-4" />}
        />
      ) : (
        <div className="space-y-2">
          {tasks.map((task) => (
            <DayTaskRow key={task.id} task={task} />
          ))}
        </div>
      )}
    </div>
  );
}

function DayTaskRow({ task }: { task: Task }) {
  const done = task.status === "completed";
  const skipped = task.status === "skipped";
  const active = task.status === "in_progress";
  const finishedAt = task.completedAt ? formatClock(task.completedAt) : null;

  return (
    <div
      className={cn(
        "flex items-start gap-2.5 px-3 py-2.5 rounded-lg border-[0.5px] transition-colors",
        active ? "border-primary/50 bg-primary/5" : "border-border bg-card",
        done || skipped ? "opacity-60" : "",
      )}
    >
      <div
        className={cn(
          "w-[18px] h-[18px] rounded-full flex items-center justify-center flex-shrink-0 mt-0.5",
          done
            ? "bg-acorn-olive"
            : active
              ? "border-[1.5px] border-primary"
              : skipped
                ? "border-[1.5px] border-dashed border-border"
                : "border-[1.5px] border-border",
        )}
      >
        {done ? <Check className="w-3 h-3 text-acorn-paper" /> : null}
        {active ? <div className="w-1.5 h-1.5 rounded-full bg-primary" /> : null}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between gap-2">
          <div
            className={cn(
              "text-[13px] font-medium text-foreground truncate",
              done || skipped ? "line-through" : "",
            )}
          >
            {task.title}
          </div>
          <PriorityBadge priority={task.priority} />
        </div>
        <div className="text-[11px] text-muted-foreground tabular-nums mt-0.5 flex items-center gap-2">
          <span className="inline-flex items-center gap-1">
            <Clock className="w-3 h-3" />
            {formatDurationMinutes(task.durationMinutes)}
          </span>
          {finishedAt ? <span>· Done at {finishedAt}</span> : null}
          {task.description ? <span className="truncate">· {task.description}</span> : null}
        </div>
      </div>
    </div>
  );
}

function EmptyState({ label, icon }: { label: string; icon: React.ReactNode }) {
  return (
    <div className="flex items-center justify-center gap-2 py-8 text-[13px] text-muted-foreground">
      {icon}
      <span>{label}</span>
    </div>
  );
}
