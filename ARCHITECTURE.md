# Architecture

This is a Tauri 2 desktop app. The Rust side owns persistence, AI, and the OS keychain. The React side owns the UI and never talks to the network directly.

```
┌──────────────────────────────────────────────────┐
│                   React + TS                     │
│  ┌────────────┐  ┌──────────────┐ ┌───────────┐  │
│  │  Stores    │  │  Components  │ │  Hooks    │  │
│  │  (Zustand) │  │  (shadcn)    │ │  (voice)  │  │
│  └─────┬──────┘  └──────┬───────┘ └─────┬─────┘  │
│        └────── invoke() / Channel ──────┘        │
└─────────────────────────┼────────────────────────┘
                          │
┌─────────────────────────┴────────────────────────┐
│                    Tauri commands                │
│  ┌────────────┐  ┌──────────────┐ ┌───────────┐  │
│  │  session   │  │     task     │ │ settings  │  │
│  │  ai        │  │   provider   │ │ keychain  │  │
│  └─────┬──────┘  └──────┬───────┘ └─────┬─────┘  │
└────────┼────────────────┼────────────────┼───────┘
         │                │                │
   ┌─────┴─────┐   ┌──────┴──────┐   ┌────┴────┐
   │  sqlx     │   │  Provider   │   │ keyring │
   │  SQLite   │   │  trait      │   │  ── ── ─│
   └───────────┘   └──────┬──────┘   │ macOS   │
                          │          │ Win     │
              ┌───────────┴──────┐   │ Linux   │
              │ Anthropic        │   └─────────┘
              │ OpenAI compat    │       OS
              │ Ollama           │   secret store
              │ Acorn Cloud      │
              └──────────────────┘
```

## Folders

| Path | What |
|---|---|
| `src/` | React frontend |
| `src/components/` | UI components (warm Acorn theme), shadcn primitives in `ui/` |
| `src/components/settings/` | Provider sidebar + per-provider config panel |
| `src/stores/` | Zustand stores (session, settings, providers) |
| `src/hooks/` | `useRecorder` for voice in |
| `src/lib/` | Tauri command wrappers (`db.ts`, `ai.ts`), formatters, `cn` helper |
| `src/types/` | TypeScript shapes mirroring Rust models |
| `src-tauri/` | Tauri shell + Rust crate |
| `src-tauri/src/db/` | sqlx pool + typed models with `FromRow` |
| `src-tauri/src/commands/` | Tauri commands, one file per domain |
| `src-tauri/src/ai/` | Provider trait + concrete provider impls |
| `src-tauri/src/ai/providers/` | Anthropic / OpenAI-compatible / Ollama / Acorn Cloud stub |
| `src-tauri/src/speech/` | SpeechProvider trait + catalogue (parallel to `ai/`) |
| `src-tauri/src/speech/providers/` | Whisper Cloud (in 1.0); macOS + Windows native land in v1.1 |
| `src-tauri/migrations/` | sqlx migration SQL |

## Data layer

`src-tauri/src/db/`. SQLite via sqlx with the `runtime-tokio-rustls`, `chrono`, and `uuid` features. The `Database` struct holds a single pool; commands access it via `tauri::State<Database>`. Migrations run on startup via `sqlx::migrate!`.

Five tables:

- `sessions` — one row per stash. Has `raw_input`, `ai_summary`, `provider_id`, `model`, timestamps.
- `tasks` — one row per task. CHECK-constrained `priority` ∈ {high, medium, low} and `status` ∈ {pending, in_progress, completed, skipped}.
- `subtasks` — nested checklist items per task (rarely used yet; LLMs sometimes emit them).
- `provider_configs` — non-secret per-provider settings: `enabled`, `custom_endpoint`, `selected_model`, `last_used_at`.
- `speech_provider_configs` — same shape as `provider_configs` but for speech-to-text providers (`enabled`, `selected_language`, `last_used_at`).
- `settings` — generic key/value bag (`active_provider`, `active_speech_provider`, `theme`, `language`).

**API keys never live here.** They go to the OS keychain via `keyring`, namespaced by the bundle identifier `app.acorn.desktop`.

## AI layer

`src-tauri/src/ai/`. A trait + a factory:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn metadata(&self) -> &ProviderMetadata;
    async fn decompose(&self, req: DecomposeRequest, events: mpsc::Sender<DecomposeEvent>)
        -> Result<DecomposeResponse, ProviderError>;
    async fn validate_credentials(&self) -> Result<(), ProviderError>;
}
```

`metadata.rs` holds a static catalog of 12 providers with display name, default model, available models, default endpoint, and an `ApiFormat` discriminator (`Anthropic | OpenAiCompatible | Ollama | Gemini | AcornCloud`).

`build_provider()` is a single match on `ApiFormat` that returns a `Box<dyn Provider>`. Adding a new OpenAI-compatible service is one row in the catalog — the concrete impl is shared.

### Streaming

`decompose` takes a `tokio::sync::mpsc::Sender<DecomposeEvent>`. Each provider:

1. Calls its API non-streaming (for MVP — true incremental parse is v1.1).
2. Strips markdown code fences if present.
3. Parses the JSON into `DecomposeResponse`.
4. Emits one `DecomposeEvent::Task` per task with a 120 ms gap (the pop-in effect).
5. Emits `DecomposeEvent::Summary` then `Done`.

The Tauri command bridges that channel to a `tauri::ipc::Channel<DecomposeEvent>` that the React side listens to.

## Frontend

Three Zustand stores hydrate on app boot:

- `useSettingsStore` — active provider, theme, language. Owns the `html.dark` class toggle.
- `useProvidersStore` — catalog + per-provider config + credential presence map. Never holds actual key bytes.
- `useSessionStore` — today's most recent session. During `stash()`, it shows optimistic preview tasks driven by the stream events, then swaps for the persisted rows once `decompose` returns.

`App.tsx` routes between three views — `input`, `stash`, `settings` — with framer-motion crossfades.

## Speech layer

`src-tauri/src/speech/` is the deliberate twin of `src-tauri/src/ai/`. Same shape:
trait + metadata catalogue + factory + concrete provider impls.

```rust
#[async_trait]
pub trait SpeechProvider: Send + Sync {
    fn metadata(&self) -> &SpeechProviderMetadata;
    async fn transcribe(
        &self,
        audio: Vec<u8>,
        mime_type: &str,
        language: Option<&str>,
    ) -> SpeechResult<TranscribeResponse>;
    async fn validate_availability(&self) -> SpeechResult<()>;
}
```

`speech::metadata::speech_provider_catalog()` lists the choices and their
`SpeechProviderStatus` (`Available` | `ComingSoon`). v1.0 ships
`whisper_openai` (Available) and `system` (ComingSoon — `SFSpeechRecognizer`
on macOS, `Windows.Media.SpeechRecognition` on Windows; both land in v1.1).

The `commands::speech::transcribe_audio` Tauri command reads
`settings.active_speech_provider`, falls back to
`default_speech_provider_id()` (currently `whisper_openai` everywhere
because `system` is ComingSoon), loads any required API key from keychain
via `SpeechProviderMetadata::api_key_provider_id` (Whisper reuses the
`openai` keychain entry), builds a provider with `build_speech_provider`,
and returns `{ text, providerId }` so the frontend can show which backend
actually transcribed.

When v1.1 flips `system` to `Available`, macOS and Windows will pick it
up automatically — no command, schema, or frontend changes required.

## Security boundaries

- API keys: keychain only. Never the database, never the filesystem, never logged.
- LLM responses: parsed strictly into typed shapes. The raw text is discarded after parse.
- Voice: audio buffers stay in memory; we send them to Whisper API and immediately drop the bytes.
- SQL: every query parameterised via `sqlx::query("…").bind(…)`. No `format!()` ever touches a query string.

## What's deliberately *not* here

- No service worker, no background sync.
- No telemetry, no analytics, no crash reporting beacons (yet — when we add it, it'll be opt-in).
- No remote update channel in v1.0. v1.1 will add Tauri's signed updater.
- No multi-window. The whole app is one webview.
