# Pepometer

A Whoop-style app for Garmin data: a Rust core and a Tauri 2 app with a
Svelte + TypeScript UI. Android first; web and iOS later.

- Spec: `docs/spec.md`. Visual reference: `design/mockup/` (open
  `index.html`; narrow windows behave like the phone).
- The UI contract is `crates/core/src/view.rs`. Its TypeScript twins in
  `app/src/lib/generated/` are generated; never edit them by hand.

## Layout

- `crates/core`: Garmin client, SQLite storage, sync, rules, view models.
- `crates/cli`: the developer CLI (`pepometer`): sign in, sync, print views.
- `app/src`: Svelte UI. `app/src-tauri`: the Tauri shell and commands.
- `app/src-tauri/gen/android`: the Android project, committed (Kotlin lives
  here: MainActivity, background sync, token vault).
- `scripts/`: emulator and build helpers.

## Environment

Run everything inside the flake's dev shell: `nix develop -c <command>`. It
provides Rust (with Android targets), tauri-cli, Node 22, JDK 17 and the
Android SDK, NDK and emulator.

- Emulator: `scripts/emulator.sh` boots `pepo35` headless as
  `emulator-5560` (`ANDROID_SERIAL` is set by the shell). Other emulators may
  run on this host; never kill processes by pattern.
- Debug APK for the emulator:
  `cd app && cargo tauri android build --debug --apk --target x86_64`, then
  `adb install -r app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`.
  The debug app id is `ca.hld.pepometer.debug`.
- Gradle needs the SDK's patched aapt2; the shell sets `GRADLE_OPTS` for it.

## Checks

- `cargo test --workspace` (also regenerates the TypeScript view types;
  commit them).
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd app && npm run check && npm run build`
- `scripts/check-secrets.sh` scans the whole history; after changing it or
  `.gitleaks.toml`, run `scripts/test-secret-scan.sh`.

## Releases

`scripts/release.sh [X.Y.Z]` publishes a signed arm64 APK as a GitHub release
(default: the next patch version); Obtainium installs and updates the app from
these releases. Run it from the main checkout on `main` after merging changes
worth shipping. It needs the release keystore in `.data/` and refuses any APK
not signed with the pinned release certificate. Never generate a new release
keystore: installed copies would reject every update.

## Rules

- Never commit Garmin data, tokens or signing keys. Local data lives in
  `.data/` (gitignored). Test fixtures are synthetic or scrubbed.
- The pre-commit hook (`.githooks/pre-commit`, enabled by the dev shell)
  blocks secrets; never bypass it with `--no-verify`. CI scans every push.
- Show Garmin's metrics. Pepometer's own math is limited to what the spec's
  Rules section lists.
- No WHOOP names, assets or copy.
- Commit with explicit paths; never amend.
- Comments explain non-obvious constraints, not history.
