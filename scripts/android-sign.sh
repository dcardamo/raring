#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .data/signed
keystore="$PWD/.data/pepometer.keystore"
password_file="$PWD/.data/pepometer.keystore.pass"
if [[ -f "$keystore" && ! -f "$password_file" ]]; then
  echo "Keystore exists without its local password file; refusing to replace it" >&2
  exit 1
fi
if [[ ! -f "$password_file" ]]; then
  umask 077
  openssl rand -hex 24 > "$password_file"
fi
export PEPOMETER_KEYSTORE_PASSWORD
PEPOMETER_KEYSTORE_PASSWORD="$(<"$password_file")"
build_tools="$(find "$ANDROID_HOME/build-tools" -mindepth 1 -maxdepth 1 | sort -V | tail -1)"
zipalign="$build_tools/zipalign"
apksigner="$build_tools/apksigner"
if [[ ! -f "$keystore" ]]; then
  keytool -genkeypair -keystore "$keystore" -alias pepometer -keyalg RSA -keysize 3072 -validity 10000 \
    -storepass:env PEPOMETER_KEYSTORE_PASSWORD -keypass:env PEPOMETER_KEYSTORE_PASSWORD -dname 'CN=Pepometer Local Release' -noprompt
  chmod 600 "$keystore"
fi
for apk in app/src-tauri/gen/android/app/build/outputs/apk/*/release/*.apk; do
  [[ -f "$apk" ]] || continue
  name="$(basename "$apk")"
  "$zipalign" -f -p 4 "$apk" ".data/signed/$name"
  "$apksigner" sign --ks "$keystore" --ks-key-alias pepometer --ks-pass env:PEPOMETER_KEYSTORE_PASSWORD --key-pass env:PEPOMETER_KEYSTORE_PASSWORD ".data/signed/$name"
  "$apksigner" verify --verbose ".data/signed/$name"
  printf 'Signed: .data/signed/%s\n' "$name"
done
keytool -list -v -keystore "$keystore" -storepass:env PEPOMETER_KEYSTORE_PASSWORD -alias pepometer | grep 'SHA256:'
