# typed: strict
# frozen_string_literal: true

cask "acorn" do
  version "1.0.0"
  sha256 "4543f0c272ce7b051beb1d371eefdb1f2a5ac9c97f3401922e21accb3a3c6e64"

  url "https://github.com/onyxcraft/acorn/releases/download/v#{version}/Acorn_#{version}_universal.dmg"
  name "Acorn"
  desc "Warm AI desktop companion that turns a messy day into focused task cards"
  homepage "https://github.com/onyxcraft/acorn"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates false
  depends_on macos: ">= :big_sur"

  app "Acorn.app"

  uninstall quit: "app.acorn.desktop"

  zap trash: [
    "~/Library/Application Support/app.acorn.desktop",
    "~/Library/Caches/app.acorn.desktop",
    "~/Library/HTTPStorages/app.acorn.desktop",
    "~/Library/Logs/Acorn",
    "~/Library/Preferences/app.acorn.desktop.plist",
    "~/Library/Saved Application State/app.acorn.desktop.savedState",
    "~/Library/WebKit/app.acorn.desktop",
  ]
end
