import { format, formatDistanceToNowStrict } from "date-fns";

export function formatToday(date: Date = new Date()): string {
  return format(date, "EEEE, MMMM d");
}

export function formatDurationMinutes(minutes: number): string {
  if (minutes < 1) return "<1 min";
  if (minutes < 60) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  const remainder = minutes % 60;
  if (remainder === 0) return `${hours} hr`;
  return `${hours}h ${remainder}m`;
}

export function formatClock(iso: string): string {
  return format(new Date(iso), "HH:mm");
}

export function formatTookSince(startIso: string, endIso: string): string {
  const start = new Date(startIso).getTime();
  const end = new Date(endIso).getTime();
  const minutes = Math.max(1, Math.round((end - start) / 60_000));
  return formatDurationMinutes(minutes);
}

export function formatElapsedSince(startIso: string): string {
  return formatDistanceToNowStrict(new Date(startIso));
}

export function formatTimerSince(startIso: string, nowMs = Date.now()): string {
  const elapsed = Math.max(0, nowMs - new Date(startIso).getTime());
  const totalSec = Math.floor(elapsed / 1000);
  const mm = String(Math.floor(totalSec / 60)).padStart(2, "0");
  const ss = String(totalSec % 60).padStart(2, "0");
  return `${mm}:${ss}`;
}
