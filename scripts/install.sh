#!/usr/bin/env bash
set -euo pipefail

REPO="onyxcraft/acorn"
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Darwin)
    suffix="_universal.dmg"
    ;;
  Linux)
    case "$ARCH" in
      x86_64) suffix="_amd64.AppImage" ;;
      aarch64) suffix="_aarch64.AppImage" ;;
      *)
        echo "Unsupported Linux architecture: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS. Visit https://github.com/$REPO/releases" >&2
    exit 1
    ;;
esac

api="https://api.github.com/repos/$REPO/releases/latest"
tag=$(curl -fsSL "$api" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)
if [ -z "$tag" ]; then
  echo "Could not determine the latest release tag." >&2
  exit 1
fi

version="${tag#v}"
download_url="https://github.com/$REPO/releases/download/$tag/Acorn_${version}${suffix}"
dest="${TMPDIR:-/tmp}/$(basename "$download_url")"

echo "Downloading Acorn $tag for $OS ($ARCH)..."
curl -fL --progress-bar "$download_url" -o "$dest"

case "$OS" in
  Darwin)
    echo "Opening installer at $dest"
    open "$dest"
    ;;
  Linux)
    chmod +x "$dest"
    echo "Saved to $dest"
    echo "Run it with: $dest"
    ;;
esac
