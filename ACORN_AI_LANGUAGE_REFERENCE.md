# Acorn AI Language Selection — Complete Reference Architecture

**Branch**: `origin/feat/port-p6-1-ai-language`  
**Base**: `origin/main`  
**Commits**: 4 total  
**Date**: May 24, 2026

---

## Commit History

```
156c5a6 feat(ai): tell the chat assistant which language the user prefers
6ec7ec6 feat(ai): inject saved shared memory into chat system prompt
ee344ae feat(ui): real Memory panel — view, edit, and clear shared cross-provider notes
353ae4c feat(ui): expand settings information architecture
```

---

## Files Changed (7 total, 464 insertions)

| File | Changes | Purpose |
|------|---------|---------|
| `src-tauri/src/commands/chat.rs` | +65 lines | Load language from settings, inject into system prompt |
| `src/lib/i18n.ts` | +70 lines | Add i18n strings for Memory, About, and new settings sections |
| `src/components/settings/settings-page.tsx` | +61 lines | Route to new panels (Memory, About, Mascot, Display, Sounds, Privacy) |
| `src/components/settings/settings-sidebar.tsx` | +67 lines | Add sidebar navigation for new settings sections |
| `src/components/settings/memory-panel.tsx` | +105 lines | **NEW** — Memory editor UI |
| `src/components/settings/about-panel.tsx` | +62 lines | **NEW** — About Acorn panel |
| `src/components/settings/placeholder-panel.tsx` | +37 lines | **NEW** — Coming Soon placeholder for future sections |

---

## Data Flow: End-to-End

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. USER SETS LANGUAGE IN UI                                     │
│    src/components/settings/general-panel.tsx                    │
│    → <select> onChange → setLanguage(value)                     │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. ZUSTAND STORE PERSISTS TO DATABASE                           │
│    src/stores/settings.ts → useSettingsStore.setLanguage()      │
│    → settings.set("language", language)                         │
│    → invoke("set_setting", { key: "language", value })          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. RUST COMMAND LAYER PERSISTS TO SQLITE                        │
│    src-tauri/src/commands/settings.rs (not shown, exists)       │
│    → INSERT/UPDATE settings table: key='language', value=<lang> │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 4. ON CHAT TURN: LOAD LANGUAGE FROM DATABASE                    │
│    src-tauri/src/commands/chat.rs                               │
│    → load_preferred_language(db.pool())                         │
│    → SELECT value FROM settings WHERE key='language'            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 5. COMPOSE SYSTEM PROMPT WITH LANGUAGE DIRECTIVE                │
│    src-tauri/src/commands/chat.rs                               │
│    → compose_system_prompt(shared_memory, preferred_language)   │
│    → Appends: "Your friend has set the app language to          │
│      [Chinese (Simplified)|Japanese|...]. Default to replying   │
│      in that language unless they switch on their own."         │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 6. PASS DYNAMIC PROMPT TO AI PROVIDER                           │
│    src-tauri/src/commands/chat.rs                               │
│    → provider.chat_turn(&chat_messages, &tools, &system_prompt)│
│    → Every provider (Anthropic, OpenAI, Ollama, etc.) gets      │
│      the language directive in the system prompt                │
└─────────────────────────────────────────────────────────────────┘
```

---

## Schema

**No new migration required.** Language lives in the existing `settings` table:

```sql
-- From 0001_initial.sql (already exists)
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Rows used:
-- key='language', value='en' | 'zh' | 'ja' | 'ko' | 'es' | 'fr' | 'de'
-- key='shared_memory', value=<user notes>
-- key='theme', value='light' | 'dark'
```

---

## Rust Command Layer

### File: `src-tauri/src/commands/chat.rs`

**Key Changes:**

1. **Load language on every chat turn** (lines 51-52):
```rust
let shared_memory = load_shared_memory(db.pool()).await;
let preferred_language = load_preferred_language(db.pool()).await;
let system_prompt =
    compose_system_prompt(shared_memory.as_deref(), preferred_language.as_deref());
```

2. **Pass dynamic prompt instead of static constant** (line 62):
```rust
// Before:
// .chat_turn(&chat_messages, &tools, CHAT_SYSTEM_PROMPT)

// After:
.chat_turn(&chat_messages, &tools, &system_prompt)
```

3. **New helper: `load_setting_value()`** (lines 151-162):
```rust
async fn load_setting_value(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    row.map(|(v,)| v)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
```

4. **New helper: `load_preferred_language()`** (lines 168-170):
```rust
async fn load_preferred_language(pool: &sqlx::SqlitePool) -> Option<String> {
    load_setting_value(pool, LANGUAGE_KEY).await
}
```

5. **Language code → display name mapping** (lines 172-199):
```rust
fn language_display(code: &str) -> &'static str {
    if code.starts_with("zh") {
        "Chinese (Simplified)"
    } else if code.starts_with("ja") {
        "Japanese"
    } else if code.starts_with("ko") {
        "Korean"
    } else if code.starts_with("es") {
        "Spanish"
    } else if code.starts_with("fr") {
        "French"
    } else if code.starts_with("de") {
        "German"
    } else {
        "English"
    }
}
```

6. **Dynamic system prompt composition** (lines 201-217):
```rust
fn compose_system_prompt(shared_memory: Option<&str>, preferred_language: Option<&str>) -> String {
    let mut out = CHAT_SYSTEM_PROMPT.to_string();
    if let Some(lang) = preferred_language {
        out.push_str("\n\nYour friend has set the app language to ");
        out.push_str(language_display(lang));
        out.push_str(". Default to replying in that language unless they switch on their own.\n");
    }
    if let Some(memory) = shared_memory {
        out.push_str(
            "\n\nLong-term notes about your friend (user-editable, stored locally, opt-out via Settings → Memory):\n",
        );
        out.push_str(memory);
        out.push('\n');
    }
    out
}
```

**Constants:**
```rust
const SHARED_MEMORY_KEY: &str = "shared_memory";
const LANGUAGE_KEY: &str = "language";
```

---

## TypeScript/Frontend Layer

### File: `src/lib/db.ts`

**Settings wrapper functions** (already exist, no changes in this branch):
```typescript
export const settings = {
  get: (key: string) => invoke<string | null>("get_setting", { key }),
  set: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  remove: (key: string) => invoke<void>("delete_setting", { key }),
};
```

These are generic key-value getters/setters that work for any setting key.

---

### File: `src/stores/settings.ts`

**Zustand store with language state:**

```typescript
interface SettingsState {
  activeProviderId: string | null;
  theme: Theme;
  language: string;  // ← Language state
  hydrated: boolean;

  hydrate: () => Promise<void>;
  setActiveProvider: (providerId: string) => Promise<void>;
  setTheme: (theme: Theme) => Promise<void>;
  setLanguage: (language: string) => Promise<void>;  // ← Setter
}

const THEME_KEY = "theme";
const LANGUAGE_KEY = "language";

function detectLanguage(): string {
  const nav = typeof navigator !== "undefined" ? navigator.language : "en";
  return nav.startsWith("zh") ? "zh" : "en";
}

export const useSettingsStore = create<SettingsState>((set) => ({
  activeProviderId: null,
  theme: "light",
  language: detectLanguage(),  // ← Default to browser language
  hydrated: false,

  hydrate: async () => {
    const [active, theme, lang] = await Promise.all([
      providers.getActive(),
      settings.get(THEME_KEY),
      settings.get(LANGUAGE_KEY),
    ]);

    set({
      activeProviderId: active,
      theme: (theme as Theme) ?? "light",
      language: lang ?? detectLanguage(),  // ← Load from DB or fallback
      hydrated: true,
    });

    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", theme === "dark");
    }
  },

  setLanguage: async (language) => {
    await settings.set(LANGUAGE_KEY, language);  // ← Persist to DB
    set({ language });  // ← Update in-memory state
  },
}));
```

---

### File: `src/lib/i18n.ts`

**New i18n strings added** (70 lines total):

```typescript
interface UiStrings {
  // ... existing strings ...
  mascotSection: string;
  displaySection: string;
  soundsSection: string;
  privacySection: string;
  memorySection: string;
  aboutSection: string;
  comingSoonBadge: string;
  aboutPanelTitle: string;
  aboutPanelSubtitle: string;
  aboutVersionLabel: string;
  aboutBuildLabel: string;
  aboutRepoLabel: string;
  aboutLicenseLabel: string;
  aboutLicenseValue: string;
  memoryPanelTitle: string;
  memoryPanelSubtitle: string;
  memoryPlaceholder: string;
  memorySaveButton: string;
  memoryClearButton: string;
  memoryHint: string;
  memorySavedToast: string;
  memoryClearedToast: string;
}

const EN: UiStrings = {
  // ... existing ...
  mascotSection: "Mascot",
  displaySection: "Display",
  soundsSection: "Sounds",
  privacySection: "Privacy",
  memorySection: "Memory",
  aboutSection: "About",
  comingSoonBadge: "SOON",
  aboutPanelTitle: "About Acorn",
  aboutPanelSubtitle: "Build info, license, and a link back to the repo.",
  aboutVersionLabel: "Version",
  aboutBuildLabel: "Build",
  aboutRepoLabel: "Repository",
  aboutLicenseLabel: "License",
  aboutLicenseValue: "MIT",
  memoryPanelTitle: "Shared memory",
  memoryPanelSubtitle:
    "Notes Acorn carries across every provider and every chat. Lives on your machine; never sent anywhere except into the next AI request.",
  memoryPlaceholder:
    "Anything you want every Acorn provider to remember about you — preferred working hours, recurring projects, the name of your dog, the cadence of your week.",
  memorySaveButton: "Save",
  memoryClearButton: "Clear",
  memoryHint: "Stored locally only.",
  memorySavedToast: "Memory saved",
  memoryClearedToast: "Memory cleared",
};

const ZH: UiStrings = {
  // ... existing ...
  mascotSection: "桌面伴侣",
  displaySection: "显示",
  soundsSection: "音效",
  privacySection: "隐私",
  memorySection: "共享记忆",
  aboutSection: "关于",
  comingSoonBadge: "即将",
  aboutPanelTitle: "关于 Acorn",
  aboutPanelSubtitle: "版本信息、协议、以及仓库链接。",
  aboutVersionLabel: "版本",
  aboutBuildLabel: "构建",
  aboutRepoLabel: "仓库",
  aboutLicenseLabel: "协议",
  aboutLicenseValue: "MIT",
  memoryPanelTitle: "共享记忆",
  memoryPanelSubtitle:
    "Acorn 在所有 provider 和所有对话之间共享的笔记。只存本地,不会送到第三方,只会拼进下一次 AI 请求里。",
  memoryPlaceholder:
    "你希望每个 provider 都记住的事:工作时段、长期项目、宠物名字、一周节奏 ——任意一段文字。",
  memorySaveButton: "保存",
  memoryClearButton: "清空",
  memoryHint: "仅存本地。",
  memorySavedToast: "记忆已保存",
  memoryClearedToast: "记忆已清空",
};
```

---

## React Components

### File: `src/components/settings/general-panel.tsx`

**Language picker in General Settings:**

```typescript
export function GeneralPanel() {
  const theme = useSettingsStore((s) => s.theme);
  const language = useSettingsStore((s) => s.language);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const setLanguage = useSettingsStore((s) => s.setLanguage);
  const t = strings(language);

  return (
    <section className="flex-1 px-8 py-7">
      <h2 className="text-[20px] font-medium text-foreground mb-1">{t.generalPanelTitle}</h2>
      <p className="text-[13px] text-muted-foreground mb-6">{t.generalPanelSubtitle}</p>

      <div className="space-y-5 max-w-xl">
        <Row label={t.themeLabel} caption={t.themeCaption} htmlFor="theme-toggle">
          <Switch
            id="theme-toggle"
            checked={theme === "dark"}
            onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
          />
        </Row>

        <Row label={t.languageLabel} caption={t.languageCaption} htmlFor="lang-select">
          <select
            id="lang-select"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            className="bg-card border-[0.5px] border-border rounded-md px-3 py-1.5 text-sm"
          >
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
        </Row>
      </div>
    </section>
  );
}
```

**Key points:**
- Language is a simple `<select>` with two options: `en` and `zh`
- `onChange` calls `setLanguage(e.target.value)` which:
  1. Persists to DB via `settings.set("language", value)`
  2. Updates Zustand store in-memory state
  3. Triggers re-render of all components using `useSettingsStore((s) => s.language)`

---

### File: `src/components/settings/memory-panel.tsx` (NEW)

**Memory editor UI:**

```typescript
import { Loader2, Save, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { settings } from "@/lib/db";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

export const SHARED_MEMORY_KEY = "shared_memory";

export function MemoryPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const [value, setValue] = useState("");
  const [savedValue, setSavedValue] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const stored = await settings.get(SHARED_MEMORY_KEY);
      const text = stored ?? "";
      setValue(text);
      setSavedValue(text);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const dirty = value !== savedValue;

  const handleSave = async () => {
    setSaving(true);
    try {
      const trimmed = value.trim();
      if (trimmed.length === 0) {
        await settings.remove(SHARED_MEMORY_KEY);
      } else {
        await settings.set(SHARED_MEMORY_KEY, trimmed);
      }
      setSavedValue(trimmed);
      setValue(trimmed);
      toast.success(t.memorySavedToast);
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    setSaving(true);
    try {
      await settings.remove(SHARED_MEMORY_KEY);
      setValue("");
      setSavedValue("");
      toast.success(t.memoryClearedToast);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-5">
        <h1 className="text-lg font-medium text-foreground">{t.memoryPanelTitle}</h1>
        <p className="text-xs text-muted-foreground mt-1">{t.memoryPanelSubtitle}</p>
      </header>

      <textarea
        value={value}
        onChange={(e) => setValue(e.target.value)}
        disabled={loading}
        placeholder={t.memoryPlaceholder}
        className="w-full max-w-2xl h-64 border-[0.5px] border-border rounded-md px-3 py-2.5 text-sm font-mono bg-card focus:outline-none focus:border-acorn-orange resize-y"
      />

      <div className="mt-3 flex items-center gap-2 max-w-2xl">
        <Button onClick={handleSave} disabled={!dirty || saving || loading} size="sm">
          {saving ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <Save className="w-3.5 h-3.5" />
          )}
          {t.memorySaveButton}
        </Button>
        <Button
          variant="outline"
          onClick={handleClear}
          disabled={saving || loading || savedValue.length === 0}
          size="sm"
        >
          <Trash2 className="w-3.5 h-3.5" />
          {t.memoryClearButton}
        </Button>
        <span className="ml-auto text-[11px] text-muted-foreground">{t.memoryHint}</span>
      </div>
    </section>
  );
}
```

**Key points:**
- Loads from `settings.get("shared_memory")` on mount
- Save/Clear use `settings.set()` / `settings.remove()`
- Empty input falls through to `remove()` so no blank rows linger
- Toast feedback on save/clear
- Dirty state tracking prevents accidental saves

---

### File: `src/components/settings/about-panel.tsx` (NEW)

```typescript
import { getName, getVersion } from "@tauri-apps/api/app";
import { ExternalLink } from "lucide-react";
import { useEffect, useState } from "react";

import { AcornLogo } from "@/components/acorn-logo";
import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

const REPO_URL = "https://github.com/onyxcraft/acorn";

export function AboutPanel() {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  const [appName, setAppName] = useState<string>("Acorn");
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    void getName().then(setAppName);
    void getVersion().then(setVersion);
  }, []);

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-6 flex items-start gap-3">
        <AcornLogo size={32} />
        <div>
          <h1 className="text-lg font-medium text-foreground">{t.aboutPanelTitle}</h1>
          <p className="text-xs text-muted-foreground mt-1">{t.aboutPanelSubtitle}</p>
        </div>
      </header>

      <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-3 text-sm max-w-md">
        <dt className="text-muted-foreground">{t.aboutVersionLabel}</dt>
        <dd className="font-mono text-foreground">
          {appName} {version || "…"}
        </dd>

        <dt className="text-muted-foreground">{t.aboutBuildLabel}</dt>
        <dd className="font-mono text-foreground/80">
          {import.meta.env.MODE === "production" ? "release" : "dev"}
        </dd>

        <dt className="text-muted-foreground">{t.aboutLicenseLabel}</dt>
        <dd className="font-mono text-foreground">{t.aboutLicenseValue}</dd>

        <dt className="text-muted-foreground">{t.aboutRepoLabel}</dt>
        <dd>
          <a
            href={REPO_URL}
            target="_blank"
            rel="noreferrer noopener"
            className="inline-flex items-center gap-1 text-acorn-orange hover:underline"
          >
            onyxcraft/acorn
            <ExternalLink className="w-3 h-3" />
          </a>
        </dd>
      </dl>
    </section>
  );
}
```

---

### File: `src/components/settings/placeholder-panel.tsx` (NEW)

```typescript
import { Sprout } from "lucide-react";

import { strings } from "@/lib/i18n";
import { useSettingsStore } from "@/stores/settings";

interface PlaceholderPanelProps {
  title: string;
  description: string;
}

export function PlaceholderPanel({ title, description }: PlaceholderPanelProps) {
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  return (
    <section className="flex-1 px-8 py-7 overflow-y-auto">
      <header className="mb-6">
        <div className="flex items-center gap-2 mb-2">
          <h1 className="text-lg font-medium text-foreground">{title}</h1>
          <span className="text-[10px] font-mono uppercase tracking-wider text-muted-foreground px-1.5 py-0.5 rounded bg-muted">
            {t.comingSoonBadge}
          </span>
        </div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </header>

      <div className="border-[0.5px] border-dashed border-border rounded-md p-8 text-center max-w-md">
        <Sprout className="w-6 h-6 text-acorn-orange/60 mx-auto mb-3" />
        <p className="text-sm text-muted-foreground">
          {language.startsWith("zh")
            ? "这部分正在路上,下次版本会落地。"
            : "Sprouting soon — landing in a future Acorn release."}
        </p>
      </div>
    </section>
  );
}
```

---

### File: `src/components/settings/settings-sidebar.tsx`

**Changes: Added new section types and sidebar rows**

```typescript
export type SettingsSection =
  | { kind: "general" }
  | { kind: "shortcuts" }
  | { kind: "mascot" }
  | { kind: "display" }
  | { kind: "sounds" }
  | { kind: "privacy" }
  | { kind: "memory" }
  | { kind: "about" }
  | { kind: "provider"; providerId: string };
```

**New sidebar rows added:**

```typescript
<SectionRow
  icon={<Sprout className="w-3.5 h-3.5" />}
  label={t.mascotSection}
  selected={selectedKey === "mascot"}
  onClick={() => onSelect({ kind: "mascot" })}
  comingSoonLabel={t.comingSoonBadge}
/>
<SectionRow
  icon={<Monitor className="w-3.5 h-3.5" />}
  label={t.displaySection}
  selected={selectedKey === "display"}
  onClick={() => onSelect({ kind: "display" })}
  comingSoonLabel={t.comingSoonBadge}
/>
<SectionRow
  icon={<Bell className="w-3.5 h-3.5" />}
  label={t.soundsSection}
  selected={selectedKey === "sounds"}
  onClick={() => onSelect({ kind: "sounds" })}
  comingSoonLabel={t.comingSoonBadge}
/>
<SectionRow
  icon={<Cookie className="w-3.5 h-3.5" />}
  label={t.privacySection}
  selected={selectedKey === "privacy"}
  onClick={() => onSelect({ kind: "privacy" })}
  comingSoonLabel={t.comingSoonBadge}
/>
<SectionRow
  icon={<ShieldCheck className="w-3.5 h-3.5" />}
  label={t.memorySection}
  selected={selectedKey === "memory"}
  onClick={() => onSelect({ kind: "memory" })}
/>
<SectionRow
  icon={<Info className="w-3.5 h-3.5" />}
  label={t.aboutSection}
  selected={selectedKey === "about"}
  onClick={() => onSelect({ kind: "about" })}
/>
```

**Updated `SectionRow` component to support "Coming Soon" badge:**

```typescript
interface SectionRowProps {
  icon: React.ReactNode;
  label: string;
  selected: boolean;
  onClick: () => void;
  comingSoonLabel?: string;
}

function SectionRow({ icon, label, selected, onClick, comingSoonLabel }: SectionRowProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "w-full text-left px-3 py-2 rounded-md text-sm transition-colors",
        selected ? "bg-acorn-orange/10 text-foreground" : "text-muted-foreground hover:bg-muted",
      )}
    >
      <span className="text-acorn-orange/80 flex-shrink-0">{icon}</span>
      <span className="flex-1 truncate">{label}</span>
      {comingSoonLabel ? (
        <span className="text-[9px] font-mono uppercase tracking-wider text-muted-foreground/70 px-1 py-px rounded bg-muted/60">
          {comingSoonLabel}
        </span>
      ) : null}
    </button>
  );
}
```

---

### File: `src/components/settings/settings-page.tsx`

**Changes: Route new sections to their panels**

```typescript
function SettingsContent({ section }: { section: SettingsSection }) {
  const catalog = useProvidersStore((s) => s.catalog);
  const language = useSettingsStore((s) => s.language);
  const t = strings(language);

  if (section.kind === "general") return <GeneralPanel />;
  if (section.kind === "shortcuts") return <ShortcutsPanel />;
  if (section.kind === "about") return <AboutPanel />;

  if (section.kind === "mascot") {
    return (
      <PlaceholderPanel
        title={t.mascotSection}
        description={
          language.startsWith("zh")
            ? "选一只桌面小伴侣,它会在你工作时陪着你。"
            : "Pick a small desktop companion that keeps you company while you work."
        }
      />
    );
  }

  if (section.kind === "display") {
    return (
      <PlaceholderPanel
        title={t.displaySection}
        description={
          language.startsWith("zh")
            ? "窗口形态 / 顶栏胶囊 / Dock 图标可见性。"
            : "Window shape, top-bar capsule, and Dock icon visibility."
        }
      />
    );
  }

  if (section.kind === "sounds") {
    return (
      <PlaceholderPanel
        title={t.soundsSection}
        description={
          language.startsWith("zh")
            ? "为每个事件挑一个声音,或者干脆静音。"
            : "Pick a sound for each event, or mute the whole thing."
        }
      />
    );
  }

  if (section.kind === "privacy") {
    return (
      <PlaceholderPanel
        title={t.privacySection}
        description={
          language.startsWith("zh")
            ? "活动采集 · 黑名单 · 一键导出 / 清空。"
            : "Activity capture, blocklists, and one-tap export or wipe."
        }
      />
    );
  }

  if (section.kind === "memory") return <MemoryPanel />;

  const provider = catalog.find((p) => p.id === section.providerId);
  if (!provider) {
    // ... existing provider fallback
  }
}
```

---

## Integration Points

### 1. **Settings Store → Database**
- `useSettingsStore.setLanguage(lang)` calls `settings.set("language", lang)`
- This invokes the Tauri command `set_setting` which persists to SQLite

### 2. **Database → Chat Command**
- On every chat turn, `load_preferred_language(db.pool())` reads from the settings table
- Returns `Option<String>` with the language code or `None`

### 3. **Language → System Prompt**
- `compose_system_prompt(shared_memory, preferred_language)` appends a directive
- The directive tells the AI model to default to the user's preferred language
- Example: "Your friend has set the app language to Chinese (Simplified). Default to replying in that language unless they switch on their own."

### 4. **System Prompt → All Providers**
- The dynamic prompt is passed to `provider.chat_turn()` instead of the static `CHAT_SYSTEM_PROMPT`
- Works for Anthropic, OpenAI, Ollama, CLI, and all OpenAI-compatible presets
- No per-provider changes needed

### 5. **UI Language → i18n**
- `useSettingsStore((s) => s.language)` is passed to `strings(language)`
- Returns the appropriate string set (EN or ZH)
- All components use `const t = strings(language)` to get localized strings

---

## Dependencies & Related Features

### Shared Memory (Commit 6ec7ec6)
- Uses the same `settings` table with key `shared_memory`
- Loaded and injected into system prompt alongside language
- Memory panel UI is in `memory-panel.tsx`

### Settings Architecture Expansion (Commit 353ae4c)
- Adds 5 new "Coming Soon" sections: Mascot, Display, Sounds, Privacy
- Uses `PlaceholderPanel` component for future features
- Sidebar now has 8 sections total (General, Shortcuts, Memory, About, + 4 Coming Soon)

### No Dependencies on Other Features
- Language selection is **self-contained**
- Does not depend on any v1.1 features
- Works with existing provider infrastructure

---

## iOS Port Checklist

For your iOS implementation, mirror this pattern:

- [ ] **Schema**: Add `language` row to settings table (or equivalent key-value store)
- [ ] **Settings Store**: Add `language: String` property and `setLanguage()` method
- [ ] **UI Component**: Create language picker (likely in `LanguageSettingsView`)
- [ ] **i18n**: Add language strings to both EN and ZH localization files
- [ ] **AI Integration**: Load language on every chat turn
- [ ] **Prompt Composition**: Append language directive to system prompt (use `language_display()` mapping)
- [ ] **Speech Provider**: Pass language code to `SpeechService.transcribe(language:)`
- [ ] **Persistence**: Ensure language persists across app restarts via settings store hydration

---

## Key Takeaways

1. **No schema migration needed** — language lives in the existing generic `settings` table
2. **Language is a first-class setting** — stored in Zustand, persisted to DB, loaded on every chat turn
3. **System prompt is dynamic** — composed on-the-fly with language + memory directives
4. **All providers inherit the language** — no per-provider changes needed
5. **UI is bilingual** — language picker in General Settings, all strings localized
6. **Memory is a sibling feature** — uses the same settings table, same composition pattern
7. **Settings architecture is expanding** — 5 new sections added (4 Coming Soon, 1 Memory)

