#!/usr/bin/env bash
# Publishes a signed arm64 APK as a GitHub release. Obtainium installs and
# updates the app from these releases.
#
# Usage, from the main checkout on main: scripts/release.sh [X.Y.Z]
# Without a version, the patch number goes up by one. Needs the release
# keystore in .data/ and an authenticated `gh`.
set -euo pipefail
cd "$(dirname "$0")/.."
[[ -n ${ANDROID_HOME:-} ]] || exec nix develop -c "$PWD/scripts/release.sh" "$@"

# Installed copies only accept updates signed with this certificate, so a
# missing keystore must never be replaced with a new one.
release_cert=9e42cfde6592819cae77cb25e75d5bb015e617aca03b4470ad7aeb8cf9c7264b
conf=app/src-tauri/tauri.conf.json
build_tools=$ANDROID_HOME/build-tools/35.0.0

die() {
  echo "release: $*" >&2
  exit 1
}

[[ $(git branch --show-current) == main ]] || die "run from main"
[[ -z $(git status --porcelain) ]] || die "commit or stash local changes first"
git fetch -q origin main
[[ $(git rev-parse HEAD) == $(git rev-parse origin/main) ]] || die "main must match origin/main"
[[ -f .data/pepometer.keystore && -f .data/pepometer.keystore.pass ]] ||
  die "release keystore missing from .data/"
scripts/check-secrets.sh || die "secret scan failed; see above"

current=$(jq -r .version "$conf")
if [[ $# -gt 0 ]]; then
  version=$1
else
  IFS=. read -r major minor patch <<<"$current"
  version=$major.$minor.$((patch + 1))
fi
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "version must look like 1.2.3"
[[ $version != "$current" && $(printf '%s\n' "$current" "$version" | sort -V | tail -1) == "$version" ]] ||
  die "version $version must be newer than $current"
tag=v$version
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null ||
  git ls-remote --exit-code -q --tags origin "refs/tags/$tag" >/dev/null; then
  die "$tag already exists"
fi
previous=$(git describe --tags --abbrev=0 2>/dev/null || true)

# Android's versionName and versionCode come from this version.
trap 'git checkout -q -- "$conf"' EXIT
jq --arg v "$version" '.version = $v' "$conf" >"$conf.tmp"
mv "$conf.tmp" "$conf"

(cd app && PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci --no-audit --no-fund)
(cd app && cargo tauri android build --apk --split-per-abi --target aarch64)
scripts/android-sign.sh

signed=.data/signed/app-arm64-release-unsigned.apk
badging=$("$build_tools/aapt2" dump badging "$signed")
[[ $badging == *"package: name='ca.hld.pepometer'"*"versionName='$version'"* ]] ||
  die "built APK is not ca.hld.pepometer $version"
cert=$("$build_tools/apksigner" verify --print-certs "$signed" |
  sed -n 's/^Signer #1 certificate SHA-256 digest: //p')
[[ $cert == "$release_cert" ]] || die "APK is signed with ${cert:-no certificate}, not the release key"
mkdir -p .data/release
apk=.data/release/pepometer-$version-arm64.apk
cp "$signed" "$apk"

git commit -q -m "Release $version" -- "$conf"
git tag -a "$tag" -m "Pepometer $version"
git push -q origin main "$tag"
if [[ -n $previous ]]; then
  notes=$(git log --no-merges --format='- %s' "$previous..HEAD~1")
else
  notes="First release."
fi
gh release create "$tag" "$apk" --verify-tag --title "Pepometer $version" --notes "${notes:-Maintenance release.}"
echo "Released $tag: $(gh release view "$tag" --json url --jq .url)"
