#!/bin/sh
# gozo installer: curl -fsSL https://raw.githubusercontent.com/PunGrumpy/gozo/main/install.sh | sh
#
# Environment:
#   GOZO_INSTALL   directory to install into (default: ~/.gozo/bin)
#   GOZO_VERSION   version tag to install, e.g. gozo-cli@0.2.0 (default: latest)
set -eu

repo="PunGrumpy/gozo"
install_dir="${GOZO_INSTALL:-$HOME/.gozo/bin}"
version="${GOZO_VERSION:-latest}"

err() { printf 'error: %s\n' "$1" >&2; exit 1; }
info() { printf '%s\n' "$1" >&2; }

os=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$os" in
  linux) os=linux ;;
  darwin) os=darwin ;;
  mingw*|msys*|cygwin*) os=windows ;;
  *) err "unsupported operating system: $os" ;;
esac

arch=$(uname -m)
case "$arch" in
  x86_64|amd64) arch=x86_64 ;;
  aarch64|arm64) arch=aarch64 ;;
  *) err "unsupported architecture: $arch" ;;
esac

asset="gozo-${os}-${arch}.tar.gz"
if [ "$version" = "latest" ]; then
  url="https://github.com/${repo}/releases/latest/download/${asset}"
else
  url="https://github.com/${repo}/releases/download/${version}/${asset}"
fi

command -v curl >/dev/null 2>&1 || err "curl is required"
command -v tar >/dev/null 2>&1 || err "tar is required"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

info "> Downloading ${asset}"
curl -fsSL "$url" -o "$tmp/$asset" || err "download failed: $url (no release for ${os}/${arch}?)"
tar -xzf "$tmp/$asset" -C "$tmp"

mkdir -p "$install_dir"
bin="gozo"; [ "$os" = windows ] && bin="gozo.exe"
mv "$tmp/$bin" "$install_dir/$bin"
chmod +x "$install_dir/$bin"

info "✓ Installed gozo to ${install_dir}/${bin}"
case ":$PATH:" in
  *":$install_dir:"*) ;;
  *)
    info ""
    info "Add gozo to your PATH:"
    case "${SHELL:-}" in
      */fish) info "  fish_add_path ${install_dir}" ;;
      */zsh)  info "  echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.zshrc" ;;
      *)      info "  echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.bashrc" ;;
    esac
    ;;
esac
info ""
info "Run 'gozo' in a Go project to get started."
