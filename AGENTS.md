# Raring

A readiness, sleep and training app for Garmin data: a Rust core and a
Tauri 2 app with a Svelte + TypeScript UI. Android first; web and iOS later.

- Spec: `docs/spec.md`. Visual reference: `design/mockup/` (open
  `index.html`; narrow windows behave like the phone).
- Backlog: `TODO.md`: Build next in priority order, then phone checks,
  real-account probes and open decisions. Read it before feature work. In the
  same change that ships, starts or reshapes an item, update it: move
  finished items to `docs/done.md` (newest first) with the commit and the
  verified endpoint facts, and add follow-ups you discover to the section
  they belong in.
- The UI contract is `crates/core/src/view.rs`. Its TypeScript twins in
  `app/src/lib/generated/` are generated; never edit them by hand.

## Talking to me

Dan reads these replies in Paseo, not a terminal. Write for that timeline.

- Refer to files as links, not as things to open: `crates/core/src/view.rs:42`
  (or `path:startLine-endLine`). Paseo makes a bare or backticked `path:line`
  clickable in the workspace. Don't say "look at the file".
- Show visual work inline with `![what it shows](path/to/shot.png)`, using a
  workspace-relative path. Prefer that to describing a screen or screenshot.
- Give reachable URLs as labelled links, `[open the mockup](http://neptune:5173/...)`.
  Never leave a bare `http(s)://host:port/...` in a reply: Paseo 0.12's markdown
  parser truncates a bare link at the host when a port follows
  (getpaseo/paseo#7440). Labelled links and `<url>` autolinks keep the whole
  URL. This is a client bug, so it applies even though the daemon here is
  0.11.4.

## Layout

- `crates/core`: Garmin client, SQLite storage, sync, rules, view models.
- `crates/cli`: the developer CLI (`raring`): sign in, sync, print views.
- `app/src`: Svelte UI. `app/src-tauri`: the Tauri shell and commands.
- `app/src-tauri/gen/android`: the Android project, committed (Kotlin lives
  here: MainActivity, background sync, token vault).
- `scripts/`: emulator and build helpers.

## Environment

Run everything inside the flake's dev shell: `nix develop -c <command>`. It
provides Rust (with Android targets), tauri-cli, Node 22, JDK 17 and the
Android SDK, NDK and emulator.

- Enter the shell from this checkout (root or a subdirectory such as `app/`).
  `ANDROID_USER_HOME` and `ANDROID_AVD_HOME` are anchored to the checkout root,
  not the entry directory; source archives use the enclosing project root.
  The private debug key is `.android-user/debug.keystore`. Preserve it for
  in-place debug updates; a certificate mismatch needs key recovery, not an
  uninstall or a newly generated key. The shell does not move or generate keys.
- Emulator: `scripts/emulator.sh` boots `pepo35` headless as
  `emulator-5560` (`ANDROID_SERIAL` is set by the shell). Other emulators may
  run on this host; never kill processes by pattern.
- Debug APK for the emulator:
  `cd app && cargo tauri android build --debug --apk --target x86_64`, then
  `adb install -r app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`.
  The debug app id is `app.raring.debug`.
- Gradle needs the SDK's patched aapt2; the shell sets `GRADLE_OPTS` for it.

## Checks

- `cargo test --workspace` (also regenerates the TypeScript view types;
  commit them).
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd app && npm run check && npm test && npm run test:ui && npm run build`
- `scripts/check-secrets.sh` scans the whole history; after changing it or
  `.gitleaks.toml`, run `scripts/test-secret-scan.sh`.

## Releases

Every push to `main` runs `.github/workflows/release.yml`, which builds,
signs and publishes the arm64 APK as a GitHub release for Obtainium.
CI chooses the next patch version from release tags (or a higher version in
`tauri.conf.json`), tags the pushed commit, and does not commit version bumps.
`scripts/release.sh [X.Y.Z]` optionally commits an explicit version bump,
pushes main and waits for the workflow; ordinary main pushes need no helper. The workflow signs with the key in the `release` environment's
secrets and refuses an APK without the pinned release certificate. Never
generate a new release keystore: installed copies would reject every update.
GitHub secrets can't be read back, so the maintainer's local
`.data/pepometer.keystore` is the only readable copy of the key.

To check the build and signing without publishing, run the Release workflow
by hand on `main` (`gh workflow run release.yml --ref main`).

## Rules

- Never commit Garmin data, tokens or signing keys. Local data lives in
  `.data/` (gitignored). Test fixtures are synthetic or scrubbed.
- The pre-commit hook (`.githooks/pre-commit`, enabled by the dev shell)
  blocks secrets; never bypass it with `--no-verify`. CI scans every push.
- Show Garmin's metrics. Raring's own math is limited to what the spec's
  Rules section lists.
- No other fitness brands' names, assets or copy.
- Commit with explicit paths; never amend.
- Comments explain non-obvious constraints, not history.
