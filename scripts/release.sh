#!/usr/bin/env bash
# Starts a release: sets the app version, commits, tags and pushes. The
# Release workflow (.github/workflows/release.yml) then builds, signs and
# publishes the APK as a GitHub release, which Obtainium installs. This
# script waits for that workflow and reports the result.
#
# Usage, from a clean main: scripts/release.sh [X.Y.Z]
# Without a version, the patch number goes up by one.
set -euo pipefail
cd "$(dirname "$0")/.."
conf=app/src-tauri/tauri.conf.json

die() {
  echo "release: $*" >&2
  exit 1
}

[[ $(git branch --show-current) == main ]] || die "run from main"
[[ -z $(git status --porcelain) ]] || die "commit or stash local changes first"
git fetch -q origin main
[[ $(git rev-parse HEAD) == $(git rev-parse origin/main) ]] || die "main must match origin/main"
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

# Android's versionName and versionCode come from this version.
jq --arg v "$version" '.version = $v' "$conf" >"$conf.tmp"
mv "$conf.tmp" "$conf"
git commit -q -m "Release $version" -- "$conf"
git tag -a "$tag" -m "Pepometer $version"
git push -q origin main "$tag"

echo "Pushed $tag; waiting for the Release workflow."
run=
for _ in $(seq 30); do
  run=$(gh run list --workflow release.yml --event push --branch "$tag" --limit 1 --json databaseId --jq '.[0].databaseId // empty')
  [[ -n $run ]] && break
  sleep 5
done
[[ -n $run ]] || die "no Release workflow run started for $tag"
gh run watch "$run" --exit-status >/dev/null || die "Release workflow failed: $(gh run view "$run" --json url --jq .url)"
echo "Released $tag: $(gh release view "$tag" --json url --jq .url)"
