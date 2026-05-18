# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Pluggable speech-to-text layer (`src-tauri/src/speech/`) that mirrors the
  existing AI provider pattern: trait + metadata catalogue + factory. The
  catalogue currently lists `whisper_openai` (Available) and `system`
  (ComingSoon — OS-native recognition lands in v1.1).
- `speech_provider_configs` SQLite table (migration `0002`) and a
  `settings.active_speech_provider` key so the user-facing choice survives
  restarts.
- New Tauri commands: `list_speech_providers`, `list_speech_provider_configs`,
  `save_speech_provider_config`, `get_active_speech_provider`,
  `set_active_speech_provider`. The existing `transcribe_audio` command now
  routes through the SpeechProvider abstraction and returns
  `TranscribeOutcome { text, providerId }`.
- TypeScript types and `lib/speech.ts` wrappers covering the new commands.

### Changed

- `ai.transcribe()` is now a thin wrapper that calls `speechProviders.transcribe`
  and unwraps the text field — the legacy contract (a `Promise<string>`) is
  preserved so `useRecorder` / `voice-button` keep working unchanged.
- `transcribe_audio` Tauri command moved from `commands::ai` to
  `commands::speech` to match the new module ownership.

## [1.0.0] - 2026-05-17

### Added

- Initial scaffolding: Tauri 2 + React 19 + TypeScript + Tailwind v4 + shadcn/ui.
- Local SQLite layer via sqlx with migration-driven schema (sessions, tasks, subtasks, provider_configs, settings).
- Multi-provider AI layer covering Anthropic, OpenAI, DeepSeek, OpenRouter, Ollama, Qwen, Moonshot, Groq, xAI, Mistral, and an Acorn Cloud placeholder.
- Whisper voice transcription using the OpenAI key.
- Settings page with provider sidebar, API key entry (keychain-backed), model picker, custom endpoint, and connection testing.
- Task input view with voice and paste helpers.
- Task cards with four lifecycle states and a live timer for the active task.
- GitHub Actions CI (frontend + Rust) and a tag-triggered release pipeline producing `.dmg` / `.AppImage` / `.exe`.
- `scripts/install.sh` one-liner installer for macOS and Linux.

[1.0.0]: https://github.com/onyxcraft/acorn/releases/tag/v1.0.0
