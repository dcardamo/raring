# Raring

[raring.app](https://raring.app)

A readiness, sleep and training app for Garmin watches: open it and see how
you're doing, why, and what to do today, from Garmin's own metrics.

Status: first test build (Android). See [docs/spec.md](docs/spec.md) and the
[mockup](design/mockup/) (sample data).

![Raring mockup: Today, Sleep, Training, Workout and Health screens](design/mockup/screens.png)

## Install

Android only for now. Install and update with
[Obtainium](https://github.com/ImranR98/Obtainium): add
`https://github.com/dcardamo/raring` as an app. Each GitHub release carries
a signed arm64 APK, and Obtainium updates the app when a new release appears.

Raring is not affiliated with Garmin. It uses Garmin Connect's
unofficial API, which Garmin can change at any time.

## How it works & privacy

Raring reads your Garmin account, not your watch. Keep Garmin Connect installed
and syncing so your watch data reaches Garmin first. Available metrics depend
on your watch.

The Android app talks directly to Garmin and keeps its downloaded data on your
phone. Nothing is sent to us: no analytics, telemetry, ads or remote crash
reporting. Sign-in uses Garmin's page; tokens are encrypted with Android
Keystore. Sign out in Settings to remove local history and your saved sign-in.
See the [privacy details](https://raring.app/privacy.html) for storage, notifications and
GitHub hosting caveats.

## Website

`site/` is the static raring.app site: no JavaScript, build step or third-party
resources. `.github/workflows/pages.yml` checks it in Chromium and deploys only
this directory to GitHub Pages on relevant main pushes. Pull requests only run
the checks. The Android release workflow remains separate (every main push
still publishes an APK).

Local check: `nix develop -c bash -c 'cd app && npm run test:site'`.
Preview: `nix develop -c python3 -m http.server 4174 --bind 0.0.0.0 --directory site`,
then open `http://neptune:4174` from the tailnet.
The sample screenshot comes from `cd app && npm run screenshots` inside the
dev shell (`.data/screens/today.png`); never replace it with personal Garmin data.
Fonts and their licenses are copied from `app/src/assets/fonts/`.

The site has no analytics, but GitHub processes ordinary hosting and download
requests; the privacy page distinguishes that from the app's local data.

## License

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE)
or the [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in Raring by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
