# Acorn 🌰

> Stash your day, one acorn at a time.

Acorn is a warm, open-source AI desktop companion. Dump a messy day into it, and it breaks the chaos into focused task cards — then walks you through them, one at a time.

[![MIT License](https://img.shields.io/badge/license-MIT-8B4513.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-B5803F.svg)](https://tauri.app)
[![Stars](https://img.shields.io/github/stars/onyxcraft/acorn?style=social)](https://github.com/onyxcraft/acorn)

[中文 README](README.zh-CN.md) · [Architecture](ARCHITECTURE.md) · [Contributing](CONTRIBUTING.md)

---

## What it does

You open Acorn, type (or speak) everything bouncing around in your head — *call the dentist, finish the slide deck, look at the launch metrics, run by the bank* — and hit **Stash it 🌰**. Acorn pings your favourite LLM, breaks the dump into ordered task cards with priorities and time estimates, and surfaces them one at a time so you can actually finish them.

It's a desktop app. Open it, work with it, close it. Data lives in a local SQLite file. API keys live in your OS keychain. Nothing leaves your machine that you didn't put there yourself.

## Highlights

- **Model-agnostic.** Ships with Anthropic, OpenAI, DeepSeek, OpenRouter, Ollama, plus seven more in the catalogue. Bring your own key — or run a local Qwen with Ollama for free. Now also: reuse your existing **`claude` / `codex` / `gemini` / `hermes` CLI logins** as a provider (zero extra API keys).
- **Native desktop.** Tauri 2 + React 19. ~10× smaller than the equivalent Electron app and feels like one.
- **Voice in.** Hold the 🎙️ button, talk, drop. Whisper transcribes into the input box.
- **Warm, not corporate.** Autumn palette designed around an actual aesthetic, not a default Tailwind theme.
- **Your data, your machine.** SQLite on disk, API keys in macOS Keychain / Windows Credential Manager / Linux Secret Service.

## Install

### Pre-built binaries

Grab the latest from [Releases](https://github.com/onyxcraft/acorn/releases):

- **macOS** — `.dmg` (Apple Silicon + Intel universal)
- **Windows** — `.exe` installer
- **Linux** — `.AppImage` or `.deb`

### One-liner (macOS / Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/onyxcraft/acorn/main/scripts/install.sh | bash
```

### Build from source

You'll need [Rust](https://rustup.rs/), [Node 20+](https://nodejs.org/), and [pnpm 10+](https://pnpm.io/).

```sh
git clone https://github.com/onyxcraft/acorn.git
cd acorn
pnpm install
pnpm tauri dev
```

## Quick start

1. Open Acorn. The first time it'll nudge you to set up a provider.
2. Settings → pick one (Anthropic Claude is the default; DeepSeek is friendliest for users in China; Ollama needs no key).
3. Paste your API key, hit **Save**, then **Make active**.
4. Back on the main view, dump the day into the textarea and hit **Stash it 🌰**.

That's it.

## Roadmap

**v1.0 (now)**

- Multi-provider AI layer with streaming decompose
- Whisper voice input
- Local SQLite + OS keychain for keys
- Today's stash + per-task lifecycle (pending / in-progress / done / skipped)
- Settings with provider sidebar and theme toggle

**v1.1 (next)**

- macOS code signing + notarisation
- Auto-update via Tauri updater
- Gemini provider (different request shape, deferred from v1.0)
- True incremental JSON parsing during decompose for faster TTFB
- Global hotkey to summon Acorn from anywhere
- Past days view (history of stashes)

**v2.0 (further out)**

- Acorn Cloud (managed hosting, no key required)
- Optional team / multi-device sync
- Calendar / Notion / Gmail integrations
- Reminders + gentle nudges

## Tech stack

- [**Tauri 2**](https://tauri.app) — desktop shell (Rust)
- [**React 19**](https://react.dev) + [**TypeScript 5.8**](https://www.typescriptlang.org/) — frontend
- [**Tailwind CSS v4**](https://tailwindcss.com) + [**shadcn/ui**](https://ui.shadcn.com) — styling
- [**sqlx**](https://github.com/launchbadge/sqlx) — typed SQLite access
- [**reqwest**](https://github.com/seanmonstar/reqwest) — HTTP for LLM APIs
- [**keyring**](https://github.com/hwchen/keyring-rs) — OS-native secret storage
- [**Zustand**](https://zustand-demo.pmnd.rs/) + [**framer-motion**](https://www.framer.com/motion/) — state + animation
- [**Biome**](https://biomejs.dev) — formatter + linter (replaces ESLint/Prettier)

## Contributing

PRs are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) for the dev setup, commit conventions, and the code style hooks you'll meet.

The short version: **Conventional Commits**, **`pnpm lint`** and **`cargo clippy -- -D warnings`** are both run in CI, no `any` in TypeScript, no `.unwrap()` in production Rust.

## License

[MIT](LICENSE) © 2026 Acorn Contributors.

## Star history

[![Star History Chart](https://api.star-history.com/svg?repos=onyxcraft/acorn&type=Date)](https://star-history.com/#onyxcraft/acorn&Date)
