# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Section-based Settings shell — sidebar now has `SETTINGS` (General /
  Shortcuts) and `MODELS` (Recommended / Local / Advanced / Coming soon)
  groups instead of a flat provider list. Theme and language live in their
  own General panel; the summon shortcut has a dedicated Shortcuts panel
  with an interactive recorder.
- macOS menu bar accessory mode — Acorn no longer appears in the Dock by
  default; it lives in the menu bar and is summoned with `⌘⇧A`. Closing
  the window keeps Acorn running.
- `acorn://` URL scheme + Apple Shortcuts / App Intents Swift package
  (`apple-shortcuts/`) so Shortcuts and (post-codesign) Siri can stash text
  into Acorn from anywhere.
- New `Cli` provider family — Acorn can now use a locally installed agent
  CLI (`claude`, `codex`, `gemini`, `hermes`) as its decompose backend by
  shelling out to the binary. Reuses the CLI's own auth, no API key needed.
  Four catalogue entries: `claude_cli`, `codex_cli`, `gemini_cli`, `hermes_cli`.
- Pluggable speech-to-text layer (`src-tauri/src/speech/`) mirroring the AI
  provider pattern: trait + metadata catalogue + factory. Currently lists
  `whisper_openai` (Available) and `system` (ComingSoon — on-device macOS
  SFSpeechRecognizer recognition lands in v1.1).
- `speech_provider_configs` SQLite table (migration `0004`) and a
  `settings.active_speech_provider` key so the user-facing choice survives
  restarts.
- Tauri commands: `list_speech_providers`, `list_speech_provider_configs`,
  `save_speech_provider_config`, `get_active_speech_provider`,
  `set_active_speech_provider`. `transcribe_audio` now routes through the
  SpeechProvider abstraction and returns `TranscribeOutcome { text, providerId }`.
- TypeScript types and `lib/speech.ts` wrappers for the new commands.
- Homebrew cask scaffold (`homebrew/`) for distribution via
  `brew install --cask acorn`.

### Changed

- `ai.transcribe()` is now a thin wrapper that calls
  `speechProviders.transcribe` and unwraps the text field — the legacy
  contract (`Promise<string>`) is preserved so `useRecorder` / voice button
  keep working unchanged.
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
