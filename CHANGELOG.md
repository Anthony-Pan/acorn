# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
