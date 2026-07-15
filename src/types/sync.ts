export type SyncProviderKind =
  | "apple_calendar"
  | "apple_reminders"
  | "google_calendar"
  | "google_tasks";

export type TargetKind = "event" | "reminder";

export type ConflictPolicy = "lww" | "local_wins" | "remote_wins";

/** A connected sync account. Presence-only: token bytes never leave the keychain. */
export interface SyncAccount {
  id: string;
  provider: SyncProviderKind;
  targetKind: TargetKind;
  accountLabel: string;
  containerId: string | null;
  containerName: string | null;
  enabled: boolean;
  hasToken: boolean;
  conflictPolicy: ConflictPolicy;
  lastSyncedAt: string | null;
  lastError: string | null;
  createdAt: string;
}

/** A selectable remote calendar / task list. */
export interface RemoteContainer {
  id: string;
  name: string;
  writable: boolean;
}

export interface SyncStatus {
  accountId: string;
  enabled: boolean;
  lastSyncedAt: string | null;
  lastError: string | null;
  pending: number;
}

/** Streamed progress from triggerSyncNow. */
export type SyncEvent =
  | { kind: "started"; accountId: string; accountLabel: string }
  | {
      kind: "pushed";
      accountId: string;
      created: number;
      updated: number;
      deleted: number;
      failed: number;
    }
  | {
      kind: "pulled";
      accountId: string;
      applied: number;
      deleted: number;
      conflicts: number;
    }
  | { kind: "finished"; accountId: string }
  | { kind: "error"; accountId: string; message: string };
