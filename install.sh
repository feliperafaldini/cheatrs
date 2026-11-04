#!/usr/bin/env bash
# Installs cheatrs without cargo: downloads a prebuilt binary from the GitHub
# release, checks its sha256, and puts it on the PATH.
#
#   curl -fsSL https://raw.githubusercontent.com/feliperafaldini/cheatrs/main/install.sh | bash
#
# Env:
#   VERSION=0.1.0        install a specific release (default: latest)
#   INSTALL_DIR=...      where the binary goes (default: ~/.local/bin)
#   CHEATRS_BASE_URL=... override the download base, for mirrors and tests
set -euo pipefail

REPO="feliperafaldini/cheatrs"
BIN="cheatrs"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

die() { printf 'cheatrs: %s\n' "$*" >&2; exit 1; }

# One asset name per supported platform, exactly as the release workflow writes it.
os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux)
    case "$arch" in
      x86_64 | amd64) asset="${BIN}-linux-x86_64" ;;
      aarch64 | arm64) die "no Linux aarch64 build yet — open an issue if you need one" ;;
      *) die "unsupported Linux architecture: $arch" ;;
    esac
    ;;
  Darwin)
    case "$arch" in
      arm64 | aarch64) asset="${BIN}-macos-arm64" ;;
      x86_64 | amd64) asset="${BIN}-macos-x86_64" ;;
      *) die "unsupported macOS architecture: $arch" ;;
    esac
    ;;
  *)
    die "unsupported platform: $os $arch (builds exist for Linux x86_64, macOS arm64 and macOS x86_64)"
    ;;
esac

if [ -n "${CHEATRS_BASE_URL:-}" ]; then
  base="$CHEATRS_BASE_URL"
elif [ "$VERSION" = latest ]; then
  base="https://github.com/$REPO/releases/latest/download"
else
  base="https://github.com/$REPO/releases/download/v${VERSION#v}"
fi

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$2" "$1"
  else
    die "need curl or wget"
  fi
}

# macOS has shasum, not sha256sum; both read the same "hash  file" line.
verify() {
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$tmp" && sha256sum -c --quiet "$asset.sha256")
  elif command -v shasum >/dev/null 2>&1; then
    (cd "$tmp" && shasum -a 256 -c "$asset.sha256" >/dev/null)
  else
    die "need sha256sum or shasum to verify the download"
  fi
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

printf 'downloading %s/%s\n' "$base" "$asset"
fetch "$base/$asset" "$tmp/$asset" || die "download failed: $base/$asset"
fetch "$base/$asset.sha256" "$tmp/$asset.sha256" || die "checksum missing for $asset"

verify || die "checksum mismatch — refusing to install"

mkdir -p "$INSTALL_DIR"
install -m 755 "$tmp/$asset" "$INSTALL_DIR/$BIN"

if ! "$INSTALL_DIR/$BIN" --validate >/dev/null 2>&1; then
  die "installed to $INSTALL_DIR/$BIN but it does not run — wrong libc or wrong arch? report it"
fi

printf 'installed %s to %s\n' "$BIN" "$INSTALL_DIR/$BIN"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) printf 'add it to your PATH:\n  export PATH="%s:$PATH"\n' "$INSTALL_DIR" ;;
esac
