import { invoke } from "@tauri-apps/api/core";

export interface ActivityEntry {
  id: string;
  kind: string;
  content: string;
  createdAt: string;
}

export interface DailyBrief {
  totalEntries: number;
  byKind: [string, number][];
  sinceIso: string;
  summary: string;
}

export const activity = {
  record: (kind: string, content: string) =>
    invoke<ActivityEntry | null>("record_activity", { kind, content }),

  listRecent: (limit?: number) =>
    invoke<ActivityEntry[]>("list_recent_activity", { limit: limit ?? null }),

  clear: () => invoke<void>("clear_activity"),

  dailyBrief: (hours?: number) => invoke<DailyBrief>("build_daily_brief", { hours: hours ?? null }),
};
