# Android integration

The Kotlin `SyncWorker` owns every Android sync, including foreground `sync_now`, hourly work and morning follow-ups. `SyncGate` holds the process-wide mutex from vault read through native sync and refreshed-token save. Native JNI uses the same `filesDir/pepometer.db` as Tauri commands and marks notification decisions sent before returning them for Kotlin to post. Desktop sync uses the file vault instead.

For emulator QA, install the debug APK, open the app, and invoke `window.__TAURI_INTERNALS__.invoke('load_demo_data')` from its Chrome WebView DevTools. This seeds 30 days of **synthetic** core raw data in the debug app's database, without signing in or creating tokens; navigate to `http://tauri.localhost/?fixture=demo#today` to bypass onboarding for screenshots. The dataset and its test fixture are compiled into the core for convenience, but the `load_demo_data` command exists **only in debug builds**. Do not use it on a debug installation with real Garmin data: seeding overwrites matching days.

`bash scripts/android-bg-test.sh` deliberately deletes only the debug app's token ciphertext, then verifies no-token notification handling and a synthetic invalid-token JNI response after force-stop. It does **not** prove live Garmin refresh or notification delivery from a successful real sync. `node scripts/android-signin-test.mjs` uses an invalid `ST-test` ticket; actual Garmin credentials and phone background operation need an on-device test.

Release signing uses a keystore and password created locally in `.data/` on first run. Keep both safe: a different certificate cannot update an installed app without uninstalling it (and losing its local data). APKs, keys and emulator screenshots in `.data/` must never be committed.
