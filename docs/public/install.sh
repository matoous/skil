#!/bin/sh
set -eu

REPO="matoous/skil"
BINARY="skil"
RELEASE_BASE_URL="https://github.com/$REPO/releases/download"
LATEST_API_URL="https://api.github.com/repos/$REPO/releases/latest"

say() {
  printf '%s\n' "$*" >&2
}

fail() {
  say "install.sh: $*"
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

download() {
  url="$1"
  out="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$out"
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    wget -qO "$out" "$url"
    return
  fi

  fail "either curl or wget is required"
}

sha256_file() {
  file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
    return
  fi

  fail "sha256sum or shasum is required for checksum verification"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) printf '%s\n' "x86_64-unknown-linux-musl" ;;
        *) fail "unsupported Linux architecture: $arch (supported: x86_64)" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64|amd64) printf '%s\n' "x86_64-apple-darwin" ;;
        arm64|aarch64) printf '%s\n' "aarch64-apple-darwin" ;;
        *) fail "unsupported macOS architecture: $arch (supported: x86_64, arm64)" ;;
      esac
      ;;
    *)
      fail "unsupported operating system: $os (supported: Linux, macOS)"
      ;;
  esac
}

resolve_tag() {
  if [ "${SKIL_VERSION:-}" ]; then
    case "$SKIL_VERSION" in
      v*) printf '%s\n' "$SKIL_VERSION" ;;
      *) printf 'v%s\n' "$SKIL_VERSION" ;;
    esac
    return
  fi

  metadata_file="$1"
  download "$LATEST_API_URL" "$metadata_file"
  tag="$(sed -n 's/^[[:space:]]*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' "$metadata_file" | head -n 1)"
  [ -n "$tag" ] || fail "failed to resolve latest release tag from GitHub API"
  printf '%s\n' "$tag"
}

install_binary() {
  source_bin="$1"
  target_bin="$2"

  if install -m 0755 "$source_bin" "$target_bin" 2>/dev/null; then
    return
  fi

  if command -v sudo >/dev/null 2>&1; then
    sudo install -m 0755 "$source_bin" "$target_bin"
    return
  fi

  fail "cannot write to $(dirname "$target_bin"); rerun with SKIL_INSTALL_DIR set to a writable path"
}

need_cmd tar

target="$(detect_target)"

tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t skil-install)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

tag="$(resolve_tag "$tmp_dir/release.json")"
version="${tag#v}"
archive="skil-${version}-${target}.tar.gz"
archive_url="$RELEASE_BASE_URL/$tag/$archive"
checksums_url="$RELEASE_BASE_URL/$tag/SHA256SUMS"

say "Installing $BINARY $tag for $target"
say "Downloading archive..."
download "$archive_url" "$tmp_dir/$archive"

say "Downloading checksums..."
download "$checksums_url" "$tmp_dir/SHA256SUMS"

expected="$(grep "  $archive\$" "$tmp_dir/SHA256SUMS" | awk '{print $1}' | head -n 1 || true)"
[ -n "$expected" ] || fail "checksum entry for $archive not found in SHA256SUMS"

actual="$(sha256_file "$tmp_dir/$archive")"
[ "$expected" = "$actual" ] || fail "checksum verification failed for $archive"

say "Extracting..."
extract_dir="$tmp_dir/extract"
mkdir -p "$extract_dir"
tar -xzf "$tmp_dir/$archive" -C "$extract_dir"
[ -f "$extract_dir/$BINARY" ] || fail "binary $BINARY not found in archive"

if [ "${SKIL_INSTALL_DIR:-}" ]; then
  install_dir="$SKIL_INSTALL_DIR"
elif [ -w "/usr/local/bin" ]; then
  install_dir="/usr/local/bin"
else
  [ "${HOME:-}" ] || fail "HOME is not set; specify SKIL_INSTALL_DIR"
  install_dir="$HOME/.local/bin"
fi

mkdir -p "$install_dir"
install_binary "$extract_dir/$BINARY" "$install_dir/$BINARY"

say "Installed: $install_dir/$BINARY"
if [ "$install_dir" = "${HOME:-}/.local/bin" ]; then
  case ":${PATH:-}:" in
    *":$install_dir:"*) ;;
    *)
      say "Warning: $install_dir is not in PATH"
      say "Add it with: export PATH=\"$install_dir:\$PATH\""
      ;;
  esac
fi

say "Done."
