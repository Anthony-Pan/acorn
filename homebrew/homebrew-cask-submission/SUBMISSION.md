# Submitting Acorn to Homebrew Cask

This is the checklist for getting `acorn` into the official
`Homebrew/homebrew-cask` repository. **Do not run this until v1.1
ships codesign + notarisation** — the official tap will reject an
unsigned app.

## Prerequisites

- [ ] Apple Developer Program membership (USD $99/year)
- [ ] "Developer ID Application" certificate installed in the macOS
      login keychain
- [ ] App-specific password for `notarytool`, stored in the keychain
      profile `acorn-notary`
- [ ] DMG built, signed, and notarised. Verify locally:
  ```sh
  codesign -dv --verbose=4 /Applications/Acorn.app
  spctl --assess --verbose --type execute /Applications/Acorn.app
  # Both must print "accepted" and reference your Team ID.
  ```
- [ ] DMG uploaded to a public GitHub Release (the tauri-action
      workflow already handles this)
- [ ] SHA256 of the new DMG ready:
  ```sh
  shasum -a 256 Acorn_<version>_universal.dmg
  ```

## Submitting

1. Fork [`Homebrew/homebrew-cask`](https://github.com/Homebrew/homebrew-cask).
2. Clone your fork and create a feature branch:
   ```sh
   git clone https://github.com/<you>/homebrew-cask
   cd homebrew-cask
   git checkout -b acorn-1.0.0
   ```
3. Copy the prepared cask into `Casks/a/acorn.rb`:
   ```sh
   cp /path/to/acorn/homebrew/homebrew-cask-submission/acorn.rb Casks/a/acorn.rb
   ```
4. Update `version` and `sha256` to match the signed/notarised release.
5. Run the local audit suite (Homebrew CI runs all of these):
   ```sh
   brew style --fix Casks/a/acorn.rb
   brew audit --cask --online --new Casks/a/acorn.rb
   brew install --cask --debug --verbose Casks/a/acorn.rb
   open /Applications/Acorn.app   # confirm zero Gatekeeper prompts
   brew uninstall --cask acorn
   ```
6. Commit and open the PR:
   ```sh
   git add Casks/a/acorn.rb
   git commit -m "Add Cask acorn"
   git push -u origin acorn-1.0.0
   ```
7. Open a PR from your fork's `acorn-1.0.0` branch into
   `Homebrew/homebrew-cask:master`. The PR template asks you to tick
   each verification step you ran above.

## During review

Maintainers usually request changes within 24-48 hours. Common asks:

- **`desc` style**: must be a single short phrase, no trailing
  punctuation, no leading article ("the"/"an").
- **`livecheck`**: prefer the simpler `strategy :github_latest`
  form (already used here).
- **`zap`**: every path must actually be created by the app. Run
  `brew zap` locally and confirm.
- **`depends_on macos`**: must match the actual minimum macOS your
  app supports. Check `MacOSXDeploymentTarget` in `Cargo.toml` and
  `tauri.conf.json`.
- **`auto_updates`**: should be `true` once Tauri's auto-updater ships
  in v1.1 (currently `false`).

If the PR gets stuck, the `#homebrew` channel on the MacAdmins Slack
is the fastest place to get a maintainer's eyes on it.

## After it merges

- `brew tap` is no longer needed: users can run `brew install --cask
  acorn` directly.
- Future version bumps go through the same PR process, but
  [`brew bump-cask-pr`](https://docs.brew.sh/Cask-Cookbook#updating-a-cask)
  automates almost everything:
  ```sh
  brew bump-cask-pr --version 1.1.0 acorn
  ```
- Consider keeping `onyxcraft/homebrew-acorn` as a beta/preview tap
  for unreleased builds even after this merges.
