#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../app"
cargo tauri android build --apk --split-per-abi --target aarch64 --target x86_64
printf 'Unsigned release APKs:\n'
find src-tauri/gen/android/app/build/outputs/apk -name '*release*.apk' -print
