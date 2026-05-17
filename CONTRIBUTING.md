# Contributing to Acorn

Thanks for thinking about making Acorn better. This is a small, opinionated project — the rules below keep it that way.

## Dev setup

You need:

- Rust stable (install via [rustup](https://rustup.rs/))
- Node 20+ ([nodejs.org](https://nodejs.org/))
- pnpm 10+ (`npm i -g pnpm` or `corepack enable`)

Linux also needs the WebKitGTK headers — the CI workflow has the apt incantation if you forget which packages.

```sh
git clone https://github.com/onyxcraft/acorn.git
cd acorn
pnpm install
pnpm tauri dev
```

The first `cargo` build downloads ~340 crates and takes a few minutes; subsequent builds are incremental.

## Local checks

Run these before you open a PR. CI runs the same set on every push.

```sh
pnpm typecheck                       # tsc --noEmit
pnpm lint                            # biome check .
pnpm build                           # vite production build
cd src-tauri && cargo fmt --check    # rust formatter
cd src-tauri && cargo clippy -- -D warnings
```

If you change SQL or Rust models, also do a quick `pnpm tauri dev` smoke test to make sure migrations apply cleanly.

## Code style

- **TypeScript.** No `any`. No `@ts-ignore`. No `console.log` in production code (warn-level rule will flag it). Path alias `@/*` resolves to `src/*`.
- **Rust.** No `.unwrap()` outside tests or the absolute startup path. Use `thiserror` for library-level errors and `anyhow` for ad-hoc context. Bind every SQL parameter, never format input into a query string.
- **CSS.** Tailwind v4 + the Acorn `--acorn-*` tokens. shadcn-managed files in `src/components/ui/` are excluded from Biome on purpose — please don't reformat them by hand.

Biome and rustfmt do the actual formatting; you don't need to fight them.

## Commits

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(ai): add openrouter provider
fix(db): close pool on shutdown so tests don't leak
docs: warn that openai's whisper is the only voice route
chore: bump tauri to 2.4
```

Scopes that already exist: `ai`, `db`, `ui`, `tauri`, `ci`, `docs`. New scope, new commit type — go ahead, but keep it short.

Body is optional but appreciated for anything non-trivial. Real reasons over restating the diff. Past tense or present tense, just pick one and stay there.

## Pull requests

- Branch from `main`, name it something like `feat/openrouter` or `fix/db-pool-leak`.
- One feature per PR. If you find yourself writing "and also" in the description, that's two PRs.
- Reference the issue if there is one. If there isn't, open one first when the change is non-obvious.
- Self-review the diff before you ask anyone else to. CI will catch the easy stuff; please catch the boring stuff yourself.

## Roadmap & non-goals

- The roadmap lives in the README. If you want to work on something that's not there, please open an issue first so we can sanity-check the scope.
- Things we are *not* doing in v1: desktop pets, language translation features, calendar/email integrations, multi-device sync. They're great ideas, just not for the first month.

## License

By contributing, you agree your code ships under the [MIT License](LICENSE).
