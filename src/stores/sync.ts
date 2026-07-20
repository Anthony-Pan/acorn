import { create } from "zustand";

import { syncAccounts, syncNow } from "@/lib/sync";
import type { RemoteContainer, SyncAccount, SyncProviderKind, SyncStatus } from "@/types/sync";

interface SyncState {
  accounts: SyncAccount[];
  statuses: Record<string, SyncStatus>;
  /** Account ids with a sync cycle currently in flight. */
  syncing: Record<string, boolean>;
  hydrated: boolean;
  error: string | null;

  hydrate: () => Promise<void>;
  connect: (provider: SyncProviderKind) => Promise<void>;
  disconnect: (accountId: string) => Promise<void>;
  listContainers: (accountId: string) => Promise<RemoteContainer[]>;
  setContainer: (accountId: string, containerId: string, containerName: string) => Promise<void>;
  setEnabled: (accountId: string, enabled: boolean) => Promise<void>;
  syncNow: (accountId?: string) => Promise<void>;
}

function upsert(accounts: SyncAccount[], account: SyncAccount): SyncAccount[] {
  const index = accounts.findIndex((a) => a.id === account.id);
  if (index === -1) return [...accounts, account];
  return accounts.map((a) => (a.id === account.id ? account : a));
}

async function refreshStatuses(): Promise<Record<string, SyncStatus>> {
  const list = await syncAccounts.status();
  const statuses: Record<string, SyncStatus> = {};
  for (const s of list) {
    statuses[s.accountId] = s;
  }
  return statuses;
}

export const useSyncStore = create<SyncState>((set, get) => ({
  accounts: [],
  statuses: {},
  syncing: {},
  hydrated: false,
  error: null,

  hydrate: async () => {
    try {
      const [accounts, statuses] = await Promise.all([syncAccounts.list(), refreshStatuses()]);
      set({ accounts, statuses, hydrated: true, error: null });
    } catch (err) {
      set({ hydrated: true, error: err instanceof Error ? err.message : String(err) });
    }
  },

  connect: async (provider) => {
    set({ error: null });
    try {
      const account = provider.startsWith("google_")
        ? await syncAccounts.connectGoogle(provider)
        : await syncAccounts.connectApple(provider);
      set((s) => ({ accounts: upsert(s.accounts, account) }));
    } catch (err) {
      set({ error: err instanceof Error ? err.message : String(err) });
      throw err;
    }
  },

  disconnect: async (accountId) => {
    await syncAccounts.disconnect(accountId);
    set((s) => ({
      accounts: s.accounts.filter((a) => a.id !== accountId),
      statuses: Object.fromEntries(Object.entries(s.statuses).filter(([id]) => id !== accountId)),
    }));
  },

  listContainers: (accountId) => syncAccounts.listContainers(accountId),

  setContainer: async (accountId, containerId, containerName) => {
    const account = await syncAccounts.setContainer(accountId, containerId, containerName);
    set((s) => ({ accounts: upsert(s.accounts, account) }));
  },

  setEnabled: async (accountId, enabled) => {
    const account = await syncAccounts.setEnabled(accountId, enabled);
    set((s) => ({ accounts: upsert(s.accounts, account) }));
  },

  syncNow: async (accountId) => {
    const targets = accountId ? [accountId] : get().accounts.map((a) => a.id);
    set((s) => ({
      error: null,
      syncing: {
        ...s.syncing,
        ...Object.fromEntries(targets.map((id) => [id, true])),
      },
    }));
    try {
      await syncNow((event) => {
        if (event.kind === "finished" || event.kind === "error") {
          set((s) => ({ syncing: { ...s.syncing, [event.accountId]: false } }));
        }
        if (event.kind === "error") {
          set({ error: event.message });
        }
      }, accountId);
    } finally {
      const [accounts, statuses] = await Promise.all([syncAccounts.list(), refreshStatuses()]);
      set((s) => ({
        accounts,
        statuses,
        syncing: {
          ...s.syncing,
          ...Object.fromEntries(targets.map((id) => [id, false])),
        },
      }));
    }
  },
}));
