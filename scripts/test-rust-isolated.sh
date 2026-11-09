#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
real_home="${HOME:?HOME must be set}"
cargo_home="${CARGO_HOME:-$real_home/.cargo}"
test_home="$(mktemp -d "${TMPDIR:-/tmp}/smartconnect-test-home.XXXXXX")"
trap 'rm -rf "$test_home"' EXIT

mkdir -p "$test_home/home" "$test_home/data" "$test_home/config" "$test_home/cache" "$test_home/state"
export HOME="$test_home/home"
export XDG_DATA_HOME="$test_home/data"
export XDG_CONFIG_HOME="$test_home/config"
export XDG_CACHE_HOME="$test_home/cache"
export XDG_STATE_HOME="$test_home/state"
export CARGO_HOME="$cargo_home"

# Keep rustup's installed toolchain available when cargo is a rustup proxy.
if [[ -n "${RUSTUP_HOME:-}" ]]; then
  export RUSTUP_HOME
elif [[ -d "$real_home/.rustup" ]]; then
  export RUSTUP_HOME="$real_home/.rustup"
fi

cargo test --locked --manifest-path "$repo_root/src-tauri/Cargo.toml" "$@"
