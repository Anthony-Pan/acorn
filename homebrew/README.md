# Homebrew Distribution

This directory holds everything needed to ship Acorn through Homebrew on
two parallel tracks:

```
homebrew/
├── tap/                          # ship-it-today path (no Apple cert required)
│   ├── Casks/acorn.rb            # the cask file for our own tap repo
│   └── README.md                 # README that goes in the tap repo
└── homebrew-cask-submission/     # official Homebrew/homebrew-cask track
    ├── acorn.rb                  # slimmed-down cask for the central tap
    └── SUBMISSION.md             # step-by-step PR guide
```

## Two tracks, one app

| Path                       | What users run                                                        | Apple signing required | Today? |
| -------------------------- | --------------------------------------------------------------------- | ---------------------- | ------ |
| Our own tap                | `brew tap onyxcraft/acorn && brew install --cask acorn`               | **No**                 | ✅      |
| Official `homebrew-cask`   | `brew install --cask acorn`                                           | **Yes** (notarised)    | ❌ (v1.1) |

The two cask files are intentionally **not identical**:

- The tap cask runs `xattr -dr com.apple.quarantine` in `postflight`
  because v1.0 is unsigned. Without it, Gatekeeper would block the
  first launch.
- The official-cask version does **not** strip quarantine because
  `homebrew-cask` requires submitted apps to already be signed and
  notarised. A `postflight` quarantine override would be rejected in
  review.

When v1.1 ships codesign + notarisation, copy the tap cask, drop the
`postflight` block, and submit it to `homebrew-cask` — the rest is
identical.

## Ship to our own tap today

Two ways: by hand once, or wire it into CI for every release.

### One-time setup

1. Create a public GitHub repo named **`homebrew-acorn`** under the
   `onyxcraft` org. The `homebrew-` prefix is required by Homebrew;
   the tap is then referenced as `onyxcraft/acorn`.
2. Copy this folder's contents into that repo:
   ```sh
   # in the new homebrew-acorn checkout
   mkdir -p Casks
   cp /path/to/acorn/homebrew/tap/Casks/acorn.rb Casks/acorn.rb
   cp /path/to/acorn/homebrew/tap/README.md README.md
   git add . && git commit -m "feat: initial cask for acorn v1.0.0"
   git push
   ```
3. Verify locally before announcing:
   ```sh
   brew tap onyxcraft/acorn
   brew install --cask acorn
   open /Applications/Acorn.app   # should launch without Gatekeeper prompt
   ```
4. (Optional) Run `brew audit --cask --online --new acorn` to catch
   common Ruby/style issues.

### Per-release maintenance

When you ship v1.x.y:

1. Edit `Casks/acorn.rb` in the tap repo:
   - bump `version`
   - update `sha256` with the new DMG hash:
     ```sh
     curl -L -o /tmp/acorn.dmg \
       "https://github.com/onyxcraft/acorn/releases/download/v1.x.y/Acorn_1.x.y_universal.dmg"
     shasum -a 256 /tmp/acorn.dmg
     ```
2. Commit + push. `livecheck` will start advertising the new version
   to anyone who runs `brew update`.

### Automated bumps (recommended)

The CI workflow at `.github/workflows/release.yml` in the main acorn
repo (added alongside this directory) opens a PR to `homebrew-acorn`
on every tag push, so step 1 above happens automatically. See the
workflow comments for the required `HOMEBREW_TAP_TOKEN` secret.

## Ship to the official Homebrew Cask

Read `homebrew-cask-submission/SUBMISSION.md`. The TL;DR:

1. Acorn must be signed with an "Apple Distribution" or "Developer ID
   Application" certificate and notarised. See Tauri's [code signing
   docs](https://tauri.app/v2/distribute/sign/macos/) for the exact
   commands.
2. Verify signing locally:
   ```sh
   codesign -dv --verbose=4 /Applications/Acorn.app
   spctl --assess --verbose /Applications/Acorn.app
   ```
3. Fork `Homebrew/homebrew-cask`, drop `acorn.rb` into `Casks/a/`,
   open a PR, respond to review comments.

The exact PR checklist is in `homebrew-cask-submission/SUBMISSION.md`.
