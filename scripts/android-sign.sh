#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .data/signed
# The path and alias must match the signing material in GitHub's release environment.
keystore="$PWD/.data/pepometer.keystore"
password_file="$PWD/.data/pepometer.keystore.pass"
if [[ ! -f "$keystore" || ! -f "$password_file" ]]; then
  echo "Existing release keystore and password required; refusing to create signing material" >&2
  exit 1
fi
export RARING_KEYSTORE_PASSWORD
RARING_KEYSTORE_PASSWORD="$(<"$password_file")"
build_tools="$(find "$ANDROID_HOME/build-tools" -mindepth 1 -maxdepth 1 | sort -V | tail -1)"
zipalign="$build_tools/zipalign"
apksigner="$build_tools/apksigner"
for apk in app/src-tauri/gen/android/app/build/outputs/apk/*/release/*.apk; do
  [[ -f "$apk" ]] || continue
  name="$(basename "$apk")"
  "$zipalign" -f -p 4 "$apk" ".data/signed/$name"
  "$apksigner" sign --ks "$keystore" --ks-key-alias pepometer --ks-pass env:RARING_KEYSTORE_PASSWORD --key-pass env:RARING_KEYSTORE_PASSWORD ".data/signed/$name"
  "$apksigner" verify --verbose ".data/signed/$name"
  printf 'Signed: .data/signed/%s\n' "$name"
done
keytool -list -v -keystore "$keystore" -storepass:env RARING_KEYSTORE_PASSWORD -alias pepometer | grep 'SHA256:'
