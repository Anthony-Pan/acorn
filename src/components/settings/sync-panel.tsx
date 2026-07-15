import { CalendarDays, ListChecks, RefreshCw, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";
import { useSyncStore } from "@/stores/sync";
import type { RemoteContainer, SyncAccount, SyncProviderKind } from "@/types/sync";

const PROVIDER_LABELS: Record<SyncProviderKind, string> = {
  apple_calendar: "Apple Calendar",
  apple_reminders: "Apple Reminders",
  google_calendar: "Google Calendar",
  google_tasks: "Google Tasks",
};

const CONNECTABLE: SyncProviderKind[] = [
  "apple_calendar",
  "apple_reminders",
  "google_calendar",
  "google_tasks",
];

export function SyncPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const accounts = useSyncStore((s) => s.accounts);
  const hydrated = useSyncStore((s) => s.hydrated);
  const hydrate = useSyncStore((s) => s.hydrate);
  const error = useSyncStore((s) => s.error);
  const connect = useSyncStore((s) => s.connect);
  const [connecting, setConnecting] = useState<SyncProviderKind | null>(null);

  useEffect(() => {
    if (!hydrated) void hydrate();
  }, [hydrated, hydrate]);

  const handleConnect = async (provider: SyncProviderKind) => {
    setConnecting(provider);
    try {
      await connect(provider);
    } catch {
      // The store surfaces the message via `error`.
    } finally {
      setConnecting(null);
    }
  };

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <h2 className="mb-1 text-[20px] font-semibold text-foreground">{t.syncPanelTitle}</h2>
      <p className="mb-6 text-[13px] text-muted-foreground">{t.syncPanelSubtitle}</p>

      {error ? (
        <div className="mb-4 max-w-xl rounded-lg border-[0.5px] border-destructive/40 bg-destructive/5 px-4 py-2.5 text-[12px] text-destructive">
          {error}
        </div>
      ) : null}

      {accounts.length === 0 ? (
        <p className="mb-6 max-w-xl text-[13px] text-muted-foreground">{t.syncNoAccounts}</p>
      ) : (
        <div className="mb-8 max-w-xl space-y-3">
          {accounts.map((account) => (
            <AccountCard key={account.id} account={account} />
          ))}
        </div>
      )}

      <h3 className="mb-2 text-[13px] font-semibold text-foreground">{t.syncAddTitle}</h3>
      <div className="grid max-w-xl grid-cols-2 gap-2">
        {CONNECTABLE.map((provider) => (
          <Button
            key={provider}
            variant="outline"
            disabled={connecting !== null}
            onClick={() => void handleConnect(provider)}
            className="justify-start gap-2"
          >
            {provider.endsWith("calendar") ? (
              <CalendarDays className="h-4 w-4 text-acorn-orange/80" />
            ) : (
              <ListChecks className="h-4 w-4 text-acorn-olive" />
            )}
            {connecting === provider ? t.syncSyncing : PROVIDER_LABELS[provider]}
          </Button>
        ))}
      </div>
    </section>
  );
}

function AccountCard({ account }: { account: SyncAccount }) {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const status = useSyncStore((s) => s.statuses[account.id]);
  const syncing = useSyncStore((s) => s.syncing[account.id] ?? false);
  const listContainers = useSyncStore((s) => s.listContainers);
  const setContainer = useSyncStore((s) => s.setContainer);
  const setEnabled = useSyncStore((s) => s.setEnabled);
  const syncNow = useSyncStore((s) => s.syncNow);
  const disconnect = useSyncStore((s) => s.disconnect);

  const [containers, setContainers] = useState<RemoteContainer[] | null>(null);
  const [containersError, setContainersError] = useState(false);

  useEffect(() => {
    let cancelled = false;
    listContainers(account.id)
      .then((list) => {
        if (!cancelled) setContainers(list.filter((c) => c.writable));
      })
      .catch(() => {
        if (!cancelled) setContainersError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [account.id, listContainers]);

  const statusLine = account.lastError
    ? account.lastError
    : account.lastSyncedAt
      ? t.syncLastSynced(new Date(account.lastSyncedAt).toLocaleString(language))
      : t.syncNeverSynced;

  return (
    <div className="divide-y-[0.5px] divide-border rounded-lg border-[0.5px] border-border bg-card shadow-[var(--shadow-card)]">
      <div className="flex items-center justify-between gap-4 px-4 py-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="truncate text-[13px] font-medium text-foreground">
              {PROVIDER_LABELS[account.provider]}
            </span>
            <span className="rounded bg-muted/60 px-1.5 py-px font-mono text-[9px] uppercase tracking-wider text-muted-foreground/80">
              {account.targetKind === "event" ? t.syncTargetEvent : t.syncTargetReminder}
            </span>
          </div>
          <div className="mt-0.5 truncate text-[11px] text-muted-foreground">
            {account.accountLabel}
          </div>
        </div>
        <div className="flex flex-shrink-0 items-center gap-1.5">
          <Button
            variant="outline"
            size="sm"
            disabled={syncing || !account.enabled}
            onClick={() => void syncNow(account.id)}
            className="gap-1.5"
          >
            <RefreshCw className={syncing ? "h-3.5 w-3.5 animate-spin" : "h-3.5 w-3.5"} />
            {syncing ? t.syncSyncing : t.syncNowButton}
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={t.syncDisconnect}
            onClick={() => void disconnect(account.id)}
          >
            <Trash2 className="h-3.5 w-3.5 text-muted-foreground" />
          </Button>
        </div>
      </div>

      <div className="flex items-center justify-between gap-6 px-4 py-3">
        <Label htmlFor={`container-${account.id}`} className="text-[13px] text-foreground">
          {t.syncContainerLabel}
        </Label>
        <select
          id={`container-${account.id}`}
          value={account.containerId ?? ""}
          disabled={containers === null || containersError}
          onChange={(e) => {
            const chosen = containers?.find((c) => c.id === e.target.value);
            if (chosen) void setContainer(account.id, chosen.id, chosen.name);
          }}
          className="max-w-[240px] rounded-md border-[0.5px] border-border bg-background px-2.5 py-1 text-[13px]"
        >
          <option value="">{t.syncContainerPlaceholder}</option>
          {(containers ?? []).map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
      </div>

      <div className="flex items-center justify-between gap-6 px-4 py-3">
        <div className="min-w-0">
          <Label htmlFor={`enabled-${account.id}`} className="text-[13px] text-foreground">
            {t.syncEnabledLabel}
          </Label>
          <div className="mt-0.5 truncate text-[11px] text-muted-foreground">
            {statusLine}
            {status && status.pending > 0 ? ` · ${t.syncPendingCount(status.pending)}` : ""}
          </div>
        </div>
        <Switch
          id={`enabled-${account.id}`}
          checked={account.enabled}
          onCheckedChange={(checked) => void setEnabled(account.id, checked)}
        />
      </div>
    </div>
  );
}
