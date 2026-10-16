#!/usr/bin/env bash
# Checks that the pre-commit hook blocks each kind of secret that
# scripts/check-secrets.sh covers, lets ordinary commits through, and that the
# history scan catches a secret committed without the hook. The fake secrets are
# generated here at run time and committed only to a throwaway repository.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
command -v gitleaks >/dev/null || exec nix develop "$repo" -c "$repo/scripts/test-secret-scan.sh"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"
git init -q
git config user.name test
git config user.email test@example.invalid
git config commit.gpgsign false
mkdir scripts
cp -r "$repo/.gitleaks.toml" "$repo/.githooks" .
cp "$repo/scripts/check-secrets.sh" scripts/
git config core.hooksPath .githooks
git add .gitleaks.toml .githooks scripts
git commit -qm setup

b64url() { base64 -w0 | tr '+/' '-_' | tr -d '='; }
random() { head -c "$1" /dev/urandom | b64url; }
jwt() {
  printf '%s.%s.%s' "$(printf '{"alg":"HS256","typ":"JWT"}' | b64url)" \
    "$(printf '{"sub":"%s","exp":1999999999}' "$(random 12)" | b64url)" "$(random 32)"
}

failures=0
# expect <allowed|blocked> <description> <path...>: stage the paths and commit.
expect() {
  local want=$1 what=$2 got
  shift 2
  git add -f -- "$@"
  if git commit -qm "$what" >/dev/null 2>&1; then
    got=allowed
  else
    got=blocked
    git reset -q
    rm -rf -- "$@"
  fi
  if [[ $got == "$want" ]]; then
    echo "PASS $what: $got"
  else
    echo "FAIL $what: $got, expected $want"
    failures=$((failures + 1))
  fi
}

echo "plain text" >notes.txt
expect allowed "ordinary file" notes.txt

printf 'token = "%s"\n' "$(jwt)" >config.toml
expect blocked "JWT (Garmin access token shape)" config.toml

printf '{"di_token": "%s", "di_refresh_token": "%s"}\n' "$(random 48)" "$(random 48)" >session.json
expect blocked "token values in JSON" session.json

echo '{}' >garmin_tokens.json
expect blocked "Garmin token file" garmin_tokens.json

head -c 2048 /dev/urandom >release.keystore
expect blocked "binary keystore" release.keystore

mkdir -p .data
echo "local only" >.data/notes.txt
expect blocked "file under .data/" .data/notes.txt

# The pathspec form agents use: commit a tracked file without `git add`.
printf 'token = "%s"\n' "$(jwt)" >>notes.txt
if git commit -qm "pathspec commit" -- notes.txt >/dev/null 2>&1; then
  echo "FAIL pathspec commit with a JWT: allowed, expected blocked"
  failures=$((failures + 1))
else
  echo "PASS pathspec commit with a JWT: blocked"
fi

git checkout -q -- notes.txt
if scripts/check-secrets.sh >/dev/null 2>&1; then
  echo "PASS history scan of clean history: passes"
else
  echo "FAIL history scan of clean history: fails"
  failures=$((failures + 1))
fi
# A commit that skipped the hook must still fail the history scan.
head -c 2048 /dev/urandom >old.keystore
git add old.keystore
git commit -qm "skip the hook" --no-verify
git rm -q old.keystore
git commit -qm "delete it again"
if scripts/check-secrets.sh >/dev/null 2>&1; then
  echo "FAIL history scan with a deleted keystore: passes, expected failure"
  failures=$((failures + 1))
else
  echo "PASS history scan with a deleted keystore: fails"
fi

((failures == 0)) || {
  echo "$failures secret-scan check(s) failed" >&2
  exit 1
}
