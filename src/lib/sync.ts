import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  RemoteContainer,
  SyncAccount,
  SyncEvent,
  SyncProviderKind,
  SyncStatus,
} from "@/types/sync";

export const syncAccounts = {
  list: () => invoke<SyncAccount[]>("list_sync_accounts"),

  connectGoogle: (provider: SyncProviderKind) =>
    invoke<SyncAccount>("connect_google_account", { provider }),

  connectApple: (provider: SyncProviderKind) =>
    invoke<SyncAccount>("connect_apple_account", { provider }),

  disconnect: (accountId: string) => invoke<void>("disconnect_sync_account", { accountId }),

  listContainers: (accountId: string) =>
    invoke<RemoteContainer[]>("list_remote_containers", { accountId }),

  setContainer: (accountId: string, containerId: string, containerName: string) =>
    invoke<SyncAccount>("set_account_container", { accountId, containerId, containerName }),

  setEnabled: (accountId: string, enabled: boolean) =>
    invoke<SyncAccount>("set_account_enabled", { accountId, enabled }),

  status: (accountId?: string) =>
    invoke<SyncStatus[]>("get_sync_status", { accountId: accountId ?? null }),
};

/** Run a sync cycle now, streaming progress events; mirrors decompose(). */
export function syncNow(onEvent: (event: SyncEvent) => void, accountId?: string): Promise<void> {
  const channel = new Channel<SyncEvent>();
  channel.onmessage = onEvent;
  return invoke<void>("trigger_sync_now", { accountId: accountId ?? null, onEvent: channel });
}
