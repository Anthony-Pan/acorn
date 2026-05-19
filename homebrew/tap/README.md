# homebrew-acorn

Homebrew tap for [Acorn](https://github.com/onyxcraft/acorn) — a warm,
open-source AI desktop companion.

## Install

```sh
brew tap onyxcraft/acorn
brew install --cask acorn
```

That's it. The cask handles the Gatekeeper quarantine flag automatically
because v1.0 is not yet code-signed (signing + notarisation is on the
v1.1 roadmap).

## Upgrade

```sh
brew update
brew upgrade --cask acorn
```

## Uninstall

```sh
# remove the app, keep your stash database
brew uninstall --cask acorn

# remove the app and wipe local data (SQLite, caches, preferences)
brew uninstall --cask --zap acorn
```

`--zap` will not remove Keychain entries. To wipe those, open Keychain
Access and search for `app.acorn.desktop`.

## What lives where

| What                  | Path                                                                     |
| --------------------- | ------------------------------------------------------------------------ |
| App bundle            | `/Applications/Acorn.app`                                                |
| SQLite database       | `~/Library/Application Support/app.acorn.desktop/acorn.db`               |
| Logs                  | `~/Library/Logs/Acorn`                                                   |
| API keys (per-provider) | macOS Keychain, service `app.acorn.desktop`                            |

## License

This tap repository is MIT-licensed, same as Acorn itself.
