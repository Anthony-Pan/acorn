import {
  addMonths,
  eachDayOfInterval,
  endOfMonth,
  endOfWeek,
  format,
  isSameDay,
  isSameMonth,
  isToday,
  startOfDay,
  startOfMonth,
  startOfWeek,
  subMonths,
} from "date-fns";

import type { Session, Task } from "@/types/db";

export function dateKey(value: Date | string): string {
  const date = typeof value === "string" ? new Date(value) : value;
  return format(date, "yyyy-MM-dd");
}

export function buildMonthGrid(cursor: Date): Date[] {
  const start = startOfWeek(startOfMonth(cursor), { weekStartsOn: 1 });
  const end = endOfWeek(endOfMonth(cursor), { weekStartsOn: 1 });
  return eachDayOfInterval({ start, end });
}

export function nextMonth(cursor: Date): Date {
  return addMonths(cursor, 1);
}

export function previousMonth(cursor: Date): Date {
  return subMonths(cursor, 1);
}

export function formatMonth(cursor: Date): string {
  return format(cursor, "MMMM yyyy");
}

export function formatDayLabel(date: Date): string {
  return format(date, "EEEE, MMMM d");
}

export interface DayStats {
  total: number;
  completed: number;
  inProgress: number;
  skipped: number;
  pending: number;
}

export function tallyTasks(tasksList: readonly Task[]): DayStats {
  const stats: DayStats = {
    total: tasksList.length,
    completed: 0,
    inProgress: 0,
    skipped: 0,
    pending: 0,
  };
  for (const task of tasksList) {
    if (task.status === "completed") stats.completed++;
    else if (task.status === "in_progress") stats.inProgress++;
    else if (task.status === "skipped") stats.skipped++;
    else stats.pending++;
  }
  return stats;
}

export function groupSessionsByDate(sessionsList: readonly Session[]): Map<string, Session[]> {
  const map = new Map<string, Session[]>();
  for (const session of sessionsList) {
    const key = dateKey(session.createdAt);
    const bucket = map.get(key);
    if (bucket) bucket.push(session);
    else map.set(key, [session]);
  }
  return map;
}

export const todayKey = (): string => dateKey(startOfDay(new Date()));

export { isSameDay, isSameMonth, isToday };
